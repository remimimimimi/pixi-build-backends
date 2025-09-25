#include "pixi-build-backend.h"

#include <stddef.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

struct autotools_generator {
    /* No state required for the minimal example */
};

static char *dup_cstr(const char *message) {
    size_t length = strlen(message) + 1;
    char *copy = (char *)malloc(length);
    if (copy == NULL) {
        return NULL;
    }
    memcpy(copy, message, length);
    return copy;
}

static char *manifest_dir_from_path(const char *manifest_path) {
    if (manifest_path == NULL || manifest_path[0] == '\0') {
        return dup_cstr(".");
    }

    /* Skip repeated "./" segments that can lead to long paths */
    while (manifest_path[0] == '.' && manifest_path[1] == '/') {
        manifest_path += 2;
    }

#ifdef _WIN32
    struct _stat st;
    if (_stat(manifest_path, &st) == 0 && (st.st_mode & _S_IFDIR)) {
        return dup_cstr(manifest_path);
    }
#else
    struct stat st;
    if (stat(manifest_path, &st) == 0 && S_ISDIR(st.st_mode)) {
        return dup_cstr(manifest_path);
    }
#endif

    const char *last_forward = strrchr(manifest_path, '/');
#ifdef _WIN32
    const char *last_back = strrchr(manifest_path, '\\');
    if (last_back != NULL && (last_forward == NULL || last_back > last_forward)) {
        last_forward = last_back;
    }
#endif

    if (last_forward == NULL) {
        return dup_cstr(".");
    }

    ptrdiff_t len = last_forward - manifest_path;
    if (len <= 0) {
        return dup_cstr(".");
    }

    char *dir = (char *)malloc((size_t)len + 1);
    if (dir == NULL) {
        return NULL;
    }

    memcpy(dir, manifest_path, (size_t)len);
    dir[len] = '\0';
    return dir;
}

static void log_message(const char *fmt, ...) {
    FILE *file = fopen("/tmp/autotools-backend.log", "a");
    if (file == NULL) {
        return;
    }
    va_list args;
    va_start(args, fmt);
    vfprintf(file, fmt, args);
    va_end(args);
    fputc('\n', file);
    fclose(file);
}

static void autotools_generator_release(Erased_t *ptr) {
    struct autotools_generator *generator = (struct autotools_generator *)ptr;
    free(generator);
}

static char *autotools_generate_recipe(
    Erased_t *ptr,
    char const *project_model_json,
    char const *config_json,
    char const *manifest_path,
    char const *host_platform,
    bool editable,
    char const *variants_json,
    GeneratedRecipeHandle_t **out_recipe
) {
    (void)ptr;
    (void)project_model_json;
    (void)config_json;
    (void)manifest_path;
    (void)host_platform;
    (void)editable;
    (void)variants_json;

    IntermediateRecipeHandle_t *recipe = pbb_intermediate_recipe_new();
    if (recipe == NULL) {
        fprintf(stderr, "autotools: failed to allocate intermediate recipe\n");
        return dup_cstr("autotools: failed to allocate intermediate recipe");
    }

    const char script[] =
        "./configure --prefix=\"$PREFIX\"\n"
        "make -j\"${PBB_PARALLEL_BUILD_JOBS:-1}\"\n"
        "make install\n";
    pbb_intermediate_recipe_set_build_script(recipe, script);

    char *source_dir = manifest_dir_from_path(manifest_path);
    if (source_dir == NULL) {
        /* logging intentionally suppressed to avoid interfering with pixi */
        pbb_intermediate_recipe_release(recipe);
        return dup_cstr("autotools: failed to determine source directory");
    }
    log_message(
        "generate_recipe manifest_path=%s source_dir=%s",
        manifest_path ? manifest_path : "(null)",
        source_dir
    );

    pbb_intermediate_recipe_clear_sources(recipe);

    pbb_intermediate_recipe_add_source_path(
        recipe,
        source_dir,
        false,
        NULL,
        false
    );
    free(source_dir);

    char *recipe_yaml = pbb_intermediate_recipe_to_yaml(recipe);
    if (recipe_yaml != NULL) {
        log_message("intermediate_recipe:\n%s", recipe_yaml);
        free(recipe_yaml);
    }

    GeneratedRecipeHandle_t *generated = pbb_generated_recipe_new_empty();
    if (generated == NULL) {
        fprintf(stderr, "autotools: failed to allocate generated recipe\n");
        pbb_intermediate_recipe_release(recipe);
        return dup_cstr("autotools: failed to allocate generated recipe");
    }

    pbb_generated_recipe_set_intermediate(generated, recipe);
    pbb_generated_recipe_add_build_glob(generated, "**");
    pbb_generated_recipe_add_metadata_glob(generated, "pixi.toml");
    *out_recipe = generated;
    log_message("generate_recipe completed successfully");
    return NULL;
}

static char *autotools_extract_input_globs(
    Erased_t *ptr,
    char const *config_json,
    char const *workdir,
    bool editable,
    char **out_globs
) {
    (void)ptr;
    (void)config_json;
    (void)workdir;
    (void)editable;

    *out_globs = NULL;
    return NULL;
}

static char *autotools_default_variants(
    Erased_t *ptr,
    char const *host_platform,
    char **out_variants
) {
    (void)ptr;
    (void)host_platform;

    *out_variants = NULL;
    return NULL;
}

int main(int argc, char **argv) {
    log_message("pixi-build-autotools backend starting (argc=%d)", argc);

    struct autotools_generator *state = (struct autotools_generator *)calloc(
        1,
        sizeof(struct autotools_generator)
    );
    if (state == NULL) {
        fprintf(stderr, "autotools: failed to allocate generator state\n");
        return EXIT_FAILURE;
    }

    CGeneratorVTable_t vtable = {
        .release_vptr = autotools_generator_release,
        .generate_recipe = autotools_generate_recipe,
        .extract_input_globs_from_build = autotools_extract_input_globs,
        .default_variants = autotools_default_variants,
    };

    VirtualPtr__Erased_ptr_CGeneratorVTable_t vptr = {
        .ptr = (Erased_t *)state,
        .vtable = vtable,
    };

    Generator_t *generator = pbb_generator_new(vptr);
    if (generator == NULL) {
        fprintf(stderr, "autotools: failed to create generator handle\n");
        autotools_generator_release((Erased_t *)state);
        return EXIT_FAILURE;
    }

    slice_ref_char_const_ptr_t args = {
        .ptr = (char const * const *)argv,
        .len = (size_t)argc,
    };

    char *error = pbb_cli_run(generator, args);
    if (error != NULL) {
        fprintf(stderr, "autotools backend failed: %s\n", error);
        log_message("pbb_cli_run returned error: %s", error);
        free(error);
        return EXIT_FAILURE;
    }
    log_message("pbb_cli_run exited normally");
    return EXIT_SUCCESS;
}
