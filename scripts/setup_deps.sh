#!/bin/bash

# Fetch and build GLFW
#
glfw_bin="engine/vendor/glfw/3.4/bin"
glfw_url=https://github.com/glfw/glfw/
glfw_tag=3.4

if [ ! -d $glfw_bin ]; then
    mkdir $glfw_bin
fi

pushd $glfw_bin
    if [ ! -d glfw ]; then
        git clone --depth 1 --branch $glfw_tag $glfw_url
    fi

    cd glfw
    cmake -S . -B build-release/ -D CMAKE_BUILD_TYPE=Release -D GLFW_BUILD_EXAMPLES=OFF -D GLFW_BUILD_TESTS=OFF -D GLFW_BUILD_DOCS=OFF -D GLFW_INSTALL=OFF -D BUILD_SHARED_LIBS=ON
    cmake --build build-release/
popd
