#----------------------------------------------------------------
# Generated CMake target import file for configuration "Debug".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "MaaFramework::MaaFramework" for configuration "Debug"
set_property(TARGET MaaFramework::MaaFramework APPEND PROPERTY IMPORTED_CONFIGURATIONS DEBUG)
set_target_properties(MaaFramework::MaaFramework PROPERTIES
  IMPORTED_LINK_DEPENDENT_LIBRARIES_DEBUG "MaaFramework::MaaUtils;opencv_core;opencv_imgproc;opencv_imgcodecs;fastdeploy_ppocr;ONNXRuntime::ONNXRuntime"
  IMPORTED_LOCATION_DEBUG "${_IMPORT_PREFIX}/bin/libMaaFramework.so"
  IMPORTED_SONAME_DEBUG "libMaaFramework.so"
  )

list(APPEND _cmake_import_check_targets MaaFramework::MaaFramework )
list(APPEND _cmake_import_check_files_for_MaaFramework::MaaFramework "${_IMPORT_PREFIX}/bin/libMaaFramework.so" )

# Import target "MaaFramework::MaaToolkit" for configuration "Debug"
set_property(TARGET MaaFramework::MaaToolkit APPEND PROPERTY IMPORTED_CONFIGURATIONS DEBUG)
set_target_properties(MaaFramework::MaaToolkit PROPERTIES
  IMPORTED_LINK_DEPENDENT_LIBRARIES_DEBUG "MaaFramework::MaaUtils;MaaFramework::MaaFramework;opencv_core;opencv_imgproc;opencv_imgcodecs"
  IMPORTED_LOCATION_DEBUG "${_IMPORT_PREFIX}/bin/libMaaToolkit.so"
  IMPORTED_SONAME_DEBUG "libMaaToolkit.so"
  )

list(APPEND _cmake_import_check_targets MaaFramework::MaaToolkit )
list(APPEND _cmake_import_check_files_for_MaaFramework::MaaToolkit "${_IMPORT_PREFIX}/bin/libMaaToolkit.so" )

# Import target "MaaFramework::MaaAgentClient" for configuration "Debug"
set_property(TARGET MaaFramework::MaaAgentClient APPEND PROPERTY IMPORTED_CONFIGURATIONS DEBUG)
set_target_properties(MaaFramework::MaaAgentClient PROPERTIES
  IMPORTED_LINK_DEPENDENT_LIBRARIES_DEBUG "opencv_core;opencv_imgproc;opencv_imgcodecs;MaaFramework::MaaUtils;MaaFramework::MaaFramework"
  IMPORTED_LOCATION_DEBUG "${_IMPORT_PREFIX}/bin/libMaaAgentClient.so"
  IMPORTED_SONAME_DEBUG "libMaaAgentClient.so"
  )

list(APPEND _cmake_import_check_targets MaaFramework::MaaAgentClient )
list(APPEND _cmake_import_check_files_for_MaaFramework::MaaAgentClient "${_IMPORT_PREFIX}/bin/libMaaAgentClient.so" )

# Import target "MaaFramework::MaaUtils" for configuration "Debug"
set_property(TARGET MaaFramework::MaaUtils APPEND PROPERTY IMPORTED_CONFIGURATIONS DEBUG)
set_target_properties(MaaFramework::MaaUtils PROPERTIES
  IMPORTED_LINK_DEPENDENT_LIBRARIES_DEBUG "opencv_core;opencv_imgproc;opencv_imgcodecs"
  IMPORTED_LOCATION_DEBUG "${_IMPORT_PREFIX}/bin/libMaaUtils.so"
  IMPORTED_SONAME_DEBUG "libMaaUtils.so"
  )

list(APPEND _cmake_import_check_targets MaaFramework::MaaUtils )
list(APPEND _cmake_import_check_files_for_MaaFramework::MaaUtils "${_IMPORT_PREFIX}/bin/libMaaUtils.so" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
