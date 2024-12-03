#!/bin/bash

echo Hello there!

srcdir="assets/shaders"
outdir="assets/shaders/.cache"

if [ ! -d $outdir ]; then
    mkdir $outdir
fi

# --target-env vulkan1.3
# --glsl-version 460
# -o <output_file>
# -t
#    multithreaded mode
# -g
#    generate debug info
# -g0
#    strip debug info
# -gV
#    generate nonsemantic shader debug info
# -gVS
#    generate nonsemantic shader debug info with source
# -e <name>
#    entry point name
# -Od
#    disables optimization
# -Os
#    optimizes SPIRV to minimize size
# -I<dir>
#    specify an include path
#
# Shader File extensions:
#     .conf   to provide a config file that replaces the default configuration
#             (see -c option below for generating a template)
#     .vert   for a vertex shader
#     .tesc   for a tessellation control shader
#     .tese   for a tessellation evaluation shader
#     .geom   for a geometry shader
#     .frag   for a fragment shader
#     .comp   for a compute shader
#     .mesh   for a mesh shader
#     .task   for a task shader
#     .rgen   for a ray generation shader
#     .rint   for a ray intersection shader
#     .rahit  for a ray any hit shader
#     .rchit  for a ray closest hit shader
#     .rmiss  for a ray miss shader
#     .rcall  for a ray callable shader
#     .glsl   for .vert.glsl, .tesc.glsl, ..., .comp.glsl compound suffixes
#     .hlsl   for .vert.hlsl, .tesc.hlsl, ..., .comp.hlsl compound suffixes

# Syntax:
# glslang [option]... [file]...

glslang --target-env vulkan1.3 --glsl-version 460 -o "$outdir/gradient.comp.spv"       "$srcdir/gradient.comp"
glslang --target-env vulkan1.3 --glsl-version 460 -o "$outdir/gradient_color.comp.spv" "$srcdir/gradient_color.comp"
glslang --target-env vulkan1.3 --glsl-version 460 -o "$outdir/sky.comp.spv"            "$srcdir/sky.comp"

# Example colored triangle with hardcoded vertices
glslang --target-env vulkan1.3 --glsl-version 450 -o "$outdir/colored_triangle.vert.spv" "$srcdir/colored_triangle.vert"
glslang --target-env vulkan1.3 --glsl-version 450 -o "$outdir/colored_triangle.frag.spv" "$srcdir/colored_triangle.frag"
