#----------------------------------------------------------------
# Generated CMake target import file for configuration "RelWithDebInfo".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "MaaAgentServer::MaaAgentServer" for configuration "RelWithDebInfo"
set_property(TARGET MaaAgentServer::MaaAgentServer APPEND PROPERTY IMPORTED_CONFIGURATIONS RELWITHDEBINFO)
set_target_properties(MaaAgentServer::MaaAgentServer PROPERTIES
  IMPORTED_LINK_DEPENDENT_LIBRARIES_RELWITHDEBINFO "opencv_core;opencv_imgproc;opencv_imgcodecs;MaaFramework::MaaUtils"
  IMPORTED_LOCATION_RELWITHDEBINFO "${_IMPORT_PREFIX}/bin/libMaaAgentServer.so"
  IMPORTED_SONAME_RELWITHDEBINFO "libMaaAgentServer.so"
  )

list(APPEND _cmake_import_check_targets MaaAgentServer::MaaAgentServer )
list(APPEND _cmake_import_check_files_for_MaaAgentServer::MaaAgentServer "${_IMPORT_PREFIX}/bin/libMaaAgentServer.so" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
