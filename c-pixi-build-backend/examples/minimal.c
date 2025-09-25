#include "pixi_build_backend.h"

#include <stdio.h>

static enum PixiStatus metadata_version(
    void *ctx,
    struct PixiOptionalString *out_value,
    struct PixiOwnedString *out_error
) {
    (void)ctx;

    static const char version[] = "1.0.0";

    if (out_value) {
        out_value->is_some = true;
        out_value->value = version;
    }

    if (out_error) {
        out_error->data = NULL;
        out_error->len = 0;
    }

    return OK;
}

int main(void) {
    struct PixiOptionalMetadataQueryFn none_callback = {false, NULL};
    struct PixiOptionalMetadataQueryFn version_callback = {true, metadata_version};

    struct PixiMetadataProvider provider = {
        .ctx = NULL,
        .name = none_callback,
        .version = version_callback,
        .homepage = none_callback,
        .license = none_callback,
        .license_file = none_callback,
        .summary = none_callback,
        .description = none_callback,
        .documentation = none_callback,
        .repository = none_callback,
    };

    PixiProjectModelV1Ptr model = NULL;
    PixiGeneratedRecipePtr recipe = NULL;
    struct PixiOwnedString err = {0};

    if (pixi_project_model_new(&model, &err) != OK) {
        const char *message = err.data ? err.data : "unknown error";
        fprintf(stderr, "pixi_project_model_new failed: %s\n", message);
        pixi_owned_string_free(err);
        return 1;
    }

    if (pixi_project_model_set_name(model, "demo-package", &err) != OK) {
        const char *message = err.data ? err.data : "unknown error";
        fprintf(stderr, "pixi_project_model_set_name failed: %s\n", message);
        pixi_owned_string_free(err);
        pixi_project_model_free(model);
        return 1;
    }

    if (pixi_project_model_set_description(model, "Example project model from C", &err) != OK) {
        const char *message = err.data ? err.data : "unknown error";
        fprintf(stderr, "pixi_project_model_set_description failed: %s\n", message);
        pixi_owned_string_free(err);
        pixi_project_model_free(model);
        return 1;
    }

    enum PixiStatus status = pixi_generated_recipe_from_model(
        model,
        &provider,
        &recipe,
        &err
    );

    if (status != OK) {
        const char *message = err.data ? err.data : "unknown error";
        fprintf(stderr, "pixi_generated_recipe_from_model failed: %s\n", message);
        pixi_owned_string_free(err);
        pixi_project_model_free(model);
        return 1;
    }

    struct PixiOwnedString recipe_json = {0};
    struct PixiOwnedString recipe_err = {0};
    status = pixi_generated_recipe_to_json(recipe, &recipe_json, &recipe_err);
    pixi_generated_recipe_free(recipe);

    if (status != OK) {
        const char *message = recipe_err.data ? recipe_err.data : "unknown error";
        fprintf(stderr, "pixi_generated_recipe_to_json failed: %s\n", message);
        pixi_owned_string_free(recipe_err);
        pixi_project_model_free(model);
        return 1;
    }

    printf("Generated recipe JSON:\n%.*s\n", (int)recipe_json.len, recipe_json.data);

    pixi_owned_string_free(recipe_json);
    pixi_owned_string_free(err);
    pixi_owned_string_free(recipe_err);
    pixi_project_model_free(model);

    return 0;
}
