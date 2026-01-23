#----------------------------------------------------------------
# Generated CMake target import file for configuration "RelWithDebInfo".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "MaaFramework::MaaFramework" for configuration "RelWithDebInfo"
set_property(TARGET MaaFramework::MaaFramework APPEND PROPERTY IMPORTED_CONFIGURATIONS RELWITHDEBINFO)
set_target_properties(MaaFramework::MaaFramework PROPERTIES
  IMPORTED_LINK_DEPENDENT_LIBRARIES_RELWITHDEBINFO "MaaFramework::MaaUtils;opencv_core;opencv_imgproc;opencv_imgcodecs;fastdeploy_ppocr;ONNXRuntime::ONNXRuntime"
  IMPORTED_LOCATION_RELWITHDEBINFO "${_IMPORT_PREFIX}/bin/libMaaFramework.so"
  IMPORTED_SONAME_RELWITHDEBINFO "libMaaFramework.so"
  )

list(APPEND _cmake_import_check_targets MaaFramework::MaaFramework )
list(APPEND _cmake_import_check_files_for_MaaFramework::MaaFramework "${_IMPORT_PREFIX}/bin/libMaaFramework.so" )

# Import target "MaaFramework::MaaToolkit" for configuration "RelWithDebInfo"
set_property(TARGET MaaFramework::MaaToolkit APPEND PROPERTY IMPORTED_CONFIGURATIONS RELWITHDEBINFO)
set_target_properties(MaaFramework::MaaToolkit PROPERTIES
  IMPORTED_LINK_DEPENDENT_LIBRARIES_RELWITHDEBINFO "MaaFramework::MaaUtils;MaaFramework::MaaFramework;opencv_core;opencv_imgproc;opencv_imgcodecs"
  IMPORTED_LOCATION_RELWITHDEBINFO "${_IMPORT_PREFIX}/bin/libMaaToolkit.so"
  IMPORTED_SONAME_RELWITHDEBINFO "libMaaToolkit.so"
  )

list(APPEND _cmake_import_check_targets MaaFramework::MaaToolkit )
list(APPEND _cmake_import_check_files_for_MaaFramework::MaaToolkit "${_IMPORT_PREFIX}/bin/libMaaToolkit.so" )

# Import target "MaaFramework::MaaAgentClient" for configuration "RelWithDebInfo"
set_property(TARGET MaaFramework::MaaAgentClient APPEND PROPERTY IMPORTED_CONFIGURATIONS RELWITHDEBINFO)
set_target_properties(MaaFramework::MaaAgentClient PROPERTIES
  IMPORTED_LINK_DEPENDENT_LIBRARIES_RELWITHDEBINFO "opencv_core;opencv_imgproc;opencv_imgcodecs;MaaFramework::MaaUtils;MaaFramework::MaaFramework"
  IMPORTED_LOCATION_RELWITHDEBINFO "${_IMPORT_PREFIX}/bin/libMaaAgentClient.so"
  IMPORTED_SONAME_RELWITHDEBINFO "libMaaAgentClient.so"
  )

list(APPEND _cmake_import_check_targets MaaFramework::MaaAgentClient )
list(APPEND _cmake_import_check_files_for_MaaFramework::MaaAgentClient "${_IMPORT_PREFIX}/bin/libMaaAgentClient.so" )

# Import target "MaaFramework::MaaUtils" for configuration "RelWithDebInfo"
set_property(TARGET MaaFramework::MaaUtils APPEND PROPERTY IMPORTED_CONFIGURATIONS RELWITHDEBINFO)
set_target_properties(MaaFramework::MaaUtils PROPERTIES
  IMPORTED_LINK_DEPENDENT_LIBRARIES_RELWITHDEBINFO "opencv_core;opencv_imgproc;opencv_imgcodecs"
  IMPORTED_LOCATION_RELWITHDEBINFO "${_IMPORT_PREFIX}/bin/libMaaUtils.so"
  IMPORTED_SONAME_RELWITHDEBINFO "libMaaUtils.so"
  )

list(APPEND _cmake_import_check_targets MaaFramework::MaaUtils )
list(APPEND _cmake_import_check_files_for_MaaFramework::MaaUtils "${_IMPORT_PREFIX}/bin/libMaaUtils.so" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
