cmake_minimum_required(VERSION 3.14)
project(TUNet VERSION 0.2.0 LANGUAGES CXX)
set(CMAKE_VISIBILITY_INLINES_HIDDEN ON)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_CXX_VISIBILITY_PRESET hidden)

add_subdirectory(corrosion)

corrosion_import_crate(MANIFEST_PATH tunet/Cargo.toml)

corrosion_install(TARGETS tunet)

add_subdirectory(tunet-gui)
