cmake_minimum_required(VERSION 3.14)
project(TUNetGui VERSION 0.1.0 LANGUAGES CXX)
set(CMAKE_VISIBILITY_INLINES_HIDDEN ON)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_CXX_VISIBILITY_PRESET hidden)

include(CheckIPOSupported)
check_ipo_supported(RESULT ENABLE_IPO OUTPUT output)
if(ENABLE_IPO)
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELEASE ON)
    message(STATUS "LTO is enabled for Release")
endif()

set(PROJECT_DIR "${CMAKE_CURRENT_LIST_DIR}/..")
set(TARGET_DIR "${PROJECT_DIR}/target")

if(DEFINED RUST_TARGET)
    set(CARGO_FLAGS "--target=${RUST_TARGET}")
    set(TARGET_DIR "${TARGET_DIR}/${RUST_TARGET}")
endif()

if(CMAKE_BUILD_TYPE MATCHES "[Rr]el")
    set(LIB_DIR "${TARGET_DIR}/release")
    list(APPEND CARGO_FLAGS "--release")
else()
    set(LIB_DIR "${TARGET_DIR}/debug")
endif()

file(GLOB RUST_SOURCES "../**/*.rs")
set(RUST_LIB_PATH ${LIB_DIR}/${CMAKE_STATIC_LIBRARY_PREFIX}tunet_model_native${CMAKE_STATIC_LIBRARY_SUFFIX})

add_custom_target(tunet-model
    COMMAND cargo build --lib ${CARGO_FLAGS}
    DEPENDS ${RUST_SOURCES}
)

add_custom_command(
    OUTPUT ${RUST_LIB_PATH}
    COMMAND ${CMAKE_COMMAND} -E echo
    WORKING_DIRECTORY ${PROJECT_DIR}
    DEPENDS tunet-model
)

find_package(QT NAMES Qt6 Qt5 COMPONENTS Widgets Charts REQUIRED)
find_package(Qt${QT_VERSION_MAJOR} COMPONENTS Widgets Charts REQUIRED)

if(WIN32)
    get_target_property(_qmake_executable Qt${QT_VERSION_MAJOR}::qmake IMPORTED_LOCATION)
    get_filename_component(_qt_bin_dir "${_qmake_executable}" DIRECTORY)
    if(CMAKE_BUILD_TYPE MATCHES "[Rr]el")
        find_program(WINDEPLOYQT_EXECUTABLE windeployqt HINTS "${_qt_bin_dir}")
    else()
        find_program(WINDEPLOYQT_EXECUTABLE windeployqt.debug.bat HINTS "${_qt_bin_dir}")
    endif()
endif()

file(GLOB HEADERS "include/*.hpp")
file(GLOB SOURCES "src/*.cpp")

add_executable(tunet-gui ${SOURCES} ${HEADERS})
target_include_directories(tunet-gui
    PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}/include
)
target_link_libraries(tunet-gui PUBLIC Qt${QT_VERSION_MAJOR}::Widgets Qt${QT_VERSION_MAJOR}::Charts ${RUST_LIB_PATH})
add_dependencies(tunet-gui tunet-model)
set_target_properties(tunet-gui PROPERTIES AUTOMOC ON qt_no_entrypoint ON)

find_package(Git)
if(Git_FOUND)
    execute_process(COMMAND ${GIT_EXECUTABLE} rev-parse HEAD
        RESULT_VARIABLE GIT_RESULT
        OUTPUT_VARIABLE GIT_HASH
    )
    if(GIT_RESULT EQUAL 0)
        string(REPLACE "\n" "" GIT_HASH ${GIT_HASH})
        string(SUBSTRING ${GIT_HASH} 0 6 GIT_HASH)
        string(CONCAT TUNET_LONG_VERSION ${PROJECT_VERSION} "-" ${GIT_HASH})
    endif()
else()
    set(TUNET_LONG_VERSION ${PROJECT_VERSION})
endif()
target_compile_definitions(tunet-gui PRIVATE "TUNET_VERSION=\"${TUNET_LONG_VERSION}\"")

if(WIN32)
    set(WINDOWS_LIB ${CMAKE_BINARY_DIR}/windows.lib)
    if(NOT EXISTS "${WINDOWS_LIB}")
        file(DOWNLOAD https://github.com/microsoft/windows-rs/raw/master/crates/targets/x86_64_msvc/lib/windows.lib ${WINDOWS_LIB})
    endif()
    target_link_libraries(tunet-gui PRIVATE ntdll ${WINDOWS_LIB})
    if(CMAKE_BUILD_TYPE MATCHES "[Rr]el")
        set_target_properties(tunet-gui PROPERTIES WIN32_EXECUTABLE ON)
        target_link_options(tunet-gui PUBLIC "/ENTRY:mainCRTStartup")
    endif()
    add_custom_command(TARGET tunet-gui POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E
            env PATH="${_qt_bin_dir}" "${WINDEPLOYQT_EXECUTABLE}"
                --no-compiler-runtime
                --no-translations
                --no-system-d3d-compiler
                --no-opengl-sw
                \"$<TARGET_FILE:tunet-gui>\"
    )
elseif(UNIX)
    if(APPLE)
        target_link_libraries(tunet-gui
            PRIVATE
                "-framework AppKit"
                "-framework CoreWLAN"
                "-framework CoreFoundation"
                "-framework Security"
                "-framework SystemConfiguration"
        )
    else()
        target_link_libraries(tunet-gui PRIVATE dl pthread)
    endif()
endif()

install(TARGETS tunet-gui DESTINATION bin)

if(WIN32)
    install(CODE [[
        file(GET_RUNTIME_DEPENDENCIES
            EXECUTABLES $<TARGET_FILE:tunet-gui>
            RESOLVED_DEPENDENCIES_VAR _r_deps
            UNRESOLVED_DEPENDENCIES_VAR _u_deps
            POST_EXCLUDE_REGEXES "system32"
        )
        foreach(_file ${_r_deps})
            file(INSTALL
                DESTINATION "${CMAKE_INSTALL_PREFIX}/bin"
                FILES "${_file}"
            )
        endforeach()
        list(LENGTH _u_deps _u_length)
        if("${_u_length}" GREATER 0)
            message(WARNING "Unresolved dependencies detected!")
        endif()
    ]])

    install(CODE "
        execute_process(
            COMMAND \"${CMAKE_COMMAND}\" -E
                env PATH=\"${_qt_bin_dir}\" \"${WINDEPLOYQT_EXECUTABLE}\"
                    --no-compiler-runtime
                    --no-translations
                    --no-system-d3d-compiler
                    --no-opengl-sw
                    \"${CMAKE_INSTALL_PREFIX}/bin/tunet-gui.exe\"
        )
    ")

    include(InstallRequiredSystemLibraries)
endif()
