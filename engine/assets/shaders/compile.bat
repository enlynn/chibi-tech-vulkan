@echo off

:: Project directory
SET HOST_DIR=%~dp0
SET HOST_DIR=%HOST_DIR:~0,-1%
echo %HOST_DIR%

SET ShaderDir=%HOST_DIR%
SET CacheDir=%HOST_DIR%\.cache

IF NOT EXIST %CacheDir% mkdir %CacheDir%

glslang --target-env vulkan1.3 --glsl-version 460 -o "%CacheDir%\gradient.comp.spv"       "%ShaderDir%\gradient.comp"
glslang --target-env vulkan1.3 --glsl-version 460 -o "%CacheDir%\gradient_color.comp.spv" "%ShaderDir%\gradient_color.comp"
glslang --target-env vulkan1.3 --glsl-version 460 -o "%CacheDir%\sky.comp.spv"            "%ShaderDir%\sky.comp"

REM Example colored triangle with hardcoded vertices
glslang --target-env vulkan1.3 --glsl-version 450 -o "%CacheDir%\colored_triangle.vert.spv" "%ShaderDir%\colored_triangle.vert"
glslang --target-env vulkan1.3 --glsl-version 450 -o "%CacheDir%\colored_triangle.frag.spv" "%ShaderDir%\colored_triangle.frag"
