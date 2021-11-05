include(CheckIPOSupported)
check_ipo_supported(RESULT ENABLE_IPO OUTPUT output)
if(ENABLE_IPO)
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELEASE ON)
    message(STATUS "LTO is enabled for Release")
endif()

option(USE_QT5 "Build against Qt5" OFF)
if(${USE_QT5})
    set(QT_VERSION_MAJOR 5)
else()
    set(QT_VERSION_MAJOR 6)
endif()

find_package(Qt${QT_VERSION_MAJOR} COMPONENTS Widgets Charts REQUIRED)

if(WIN32)
    get_target_property(_qmake_executable Qt${QT_VERSION_MAJOR}::qmake IMPORTED_LOCATION)
    get_filename_component(_qt_bin_dir "${_qmake_executable}" DIRECTORY)
    if(CMAKE_BUILD_TYPE MATCHES "[Rr]el")
        find_program(WINDEPLOYQT_EXECUTABLE windeployqt HINTS "${_qt_bin_dir}")
    else()
        find_program(WINDEPLOYQT_EXECUTABLE windeployqt.debug.bat HINTS "${_qt_bin_dir}")
    endif()
elseif(APPLE)
    get_target_property(_qmake_executable Qt${QT_VERSION_MAJOR}::qmake IMPORTED_LOCATION)
    get_filename_component(_qt_bin_dir "${_qmake_executable}" DIRECTORY)
    find_program(MACDEPLOYQT_EXECUTABLE macdeployqt HINTS "${_qt_bin_dir}")
endif()

file(GLOB HEADERS "include/*.hpp")
file(GLOB SOURCES "src/*.cpp")

add_executable(tunet-gui ${SOURCES} ${HEADERS})
target_include_directories(tunet-gui
    PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}/include
)
target_link_libraries(tunet-gui PUBLIC Qt${QT_VERSION_MAJOR}::Widgets Qt${QT_VERSION_MAJOR}::Charts tunet-model-native-static)
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
    target_link_libraries(tunet-gui PUBLIC ntdll ole32 oleaut32 ws2_32 bcrypt advapi32 userenv iphlpapi secur32 crypt32 ncrypt shell32)
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
elseif(APPLE)
    target_link_libraries(tunet-gui
        PUBLIC
            "-framework AppKit"
            "-framework CoreWLAN"
            "-framework CoreFoundation"
            "-framework Security"
            "-framework SystemConfiguration"
    )
    set_target_properties(tunet-gui PROPERTIES
        MACOSX_BUNDLE ON
        MACOSX_BUNDLE_SHORT_VERSION_STRING ${PROJECT_VERSION}
        MACOSX_BUNDLE_LONG_VERSION_STRING ${TUNET_LONG_VERSION}
    )
elseif(UNIX)
    target_link_libraries(tunet-gui PUBLIC dl pthread)
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
elseif(APPLE)
    install(CODE "
        execute_process(
            COMMAND \"${MACDEPLOYQT_EXECUTABLE}\"
                \"${CMAKE_INSTALL_PREFIX}/bin/tunet-gui.app\"
                -always-overwrite
        )
    ")
elseif(UNIX)
    set(CPACK_DEBIAN_PACKAGE_MAINTAINER "Berrysoft")
    set(CPACK_DEBIAN_PACKAGE_DESCRIPTION "Tsinghua campus network client.")
    set(CPACK_DEBIAN_PACKAGE_VERSION ${PROJECT_VERSION})
    set(CPACK_DEBIAN_FILE_NAME DEB-DEFAULT)
    set(CPACK_DEBIAN_PACKAGE_DEPENDS "libc6 (>= 2.18), libqt5charts5 (>= 5.0.0)")

    include(CPack)
endif()
