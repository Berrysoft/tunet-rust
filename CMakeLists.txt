cmake_minimum_required(VERSION 3.14)
project(TUNet VERSION 0.6.3 LANGUAGES CXX)
set(CMAKE_VISIBILITY_INLINES_HIDDEN ON)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_CXX_VISIBILITY_PRESET hidden)

option(USE_QT5 "Build against Qt5" OFF)
option(USE_CORROSION "Build rust part with Corrosion" ON)
option(USE_DEPLOYQT "Use *deployqt for deployment" ON)

if(${USE_QT5})
    set(QT_VERSION_MAJOR 5)
else()
    set(QT_VERSION_MAJOR 6)
endif()

find_package(Qt${QT_VERSION_MAJOR} COMPONENTS Widgets Charts REQUIRED)

if(${USE_CORROSION})
    add_subdirectory(corrosion)

    set(TUNET_CRATES tunet tunet-cui tunet-model-native)
    if(WIN32)
        list(APPEND TUNET_CRATES tunet-service)
    endif()

    corrosion_import_crate(MANIFEST_PATH tunet/Cargo.toml CRATES ${TUNET_CRATES})

    corrosion_install(TARGETS tunet tunet-cui)
    if(WIN32)
        corrosion_install(TARGETS tunet-service)
    endif()
else()
    add_library(tunet-model-native-static STATIC IMPORTED)
    set_property(TARGET tunet-model-native-static PROPERTY IMPORTED_LOCATION ${TUNET_MODEL_NATIVE_LIB_PATH})
endif()

add_subdirectory(tunet-gui)
