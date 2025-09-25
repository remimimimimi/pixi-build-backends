cmake_minimum_required(VERSION 3.20...3.27)

project(pixi-build-autotools C)

set(CMAKE_FIND_LIBRARY_SUFFIXES ".a" ".so")

find_path(pixi_build_backend_INCLUDE_DIR
    NAMES pixi-build-backend.h
)
if(NOT pixi_build_backend_INCLUDE_DIR)
    message(FATAL_ERROR "pixi-build-backend.h not found on standard include paths.")
endif()

find_library(pixi_build_backend_LIBRARY
    NAMES pixi-build-backend libc_pixi_build_backend c_pixi_build_backend
)
if(NOT pixi_build_backend_LIBRARY)
    message(FATAL_ERROR "pixi_build_backend library not found on standard search paths.")
endif()

add_library(pixi_build_backend STATIC IMPORTED)
set_target_properties(pixi_build_backend PROPERTIES
    IMPORTED_LOCATION "${pixi_build_backend_LIBRARY}"
    INTERFACE_INCLUDE_DIRECTORIES "${pixi_build_backend_INCLUDE_DIR}"
)

add_executable(pixi-build-autotools src/main.c)
target_link_libraries(pixi-build-autotools PRIVATE pixi_build_backend m dl pthread lzma z ssl crypto)

include(GNUInstallDirs)
install(TARGETS pixi-build-autotools
    RUNTIME DESTINATION "${CMAKE_INSTALL_BINDIR}"
)
