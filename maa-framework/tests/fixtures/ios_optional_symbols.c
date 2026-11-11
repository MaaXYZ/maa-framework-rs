/* Device-free ABI fixture for older and incomplete MaaFramework libraries. */
#include <stdint.h>
#include <stdlib.h>

typedef uint8_t MaaBool;
typedef uint64_t MaaSize;
typedef uint64_t MaaIOSScreencapMethod;
typedef int32_t MaaIOSConnectionMethod;
typedef struct MaaController MaaController;
typedef struct MaaToolkitIOSDeviceList MaaToolkitIOSDeviceList;
typedef struct MaaToolkitIOSDevice MaaToolkitIOSDevice;
typedef MaaBool (*MaaToolkitIOSDeviceCancelCallback)(void *);

const char *MaaVersion(void) { return "5.15.0"; }

#ifndef OLD_SDK
static uint64_t allocations, live_handles;
uint64_t MockAllocations(void) { return allocations; }
uint64_t MockLiveHandles(void) { return live_handles; }

static void *create_handle(void)
{
    ++allocations;
    ++live_handles;
    return malloc(1);
}

static void destroy_handle(void *handle)
{
    --live_handles;
    free(handle);
}

MaaController *MaaIOSControllerCreate(const char *udid, MaaIOSScreencapMethod methods,
                                    MaaIOSConnectionMethod connection, const char *config)
{
    return create_handle();
}
void MaaControllerDestroy(MaaController *handle) { destroy_handle(handle); }
void MaaControllerClearSinks(MaaController *handle) {}

MaaToolkitIOSDeviceList *MaaToolkitIOSDeviceListCreate(void) { return create_handle(); }
void MaaToolkitIOSDeviceListDestroy(MaaToolkitIOSDeviceList *list) { destroy_handle(list); }
MaaBool MaaToolkitIOSDeviceFind(MaaToolkitIOSDeviceList *list) { return 1; }
MaaBool MaaToolkitIOSDeviceFindWithOptions(MaaIOSConnectionMethod connection,
    const char *options, MaaToolkitIOSDeviceCancelCallback cancel, void *userdata,
    MaaToolkitIOSDeviceList *list) { return 1; }
MaaSize MaaToolkitIOSDeviceListSize(const MaaToolkitIOSDeviceList *list) { return 1; }
const MaaToolkitIOSDevice *MaaToolkitIOSDeviceListAt(const MaaToolkitIOSDeviceList *list,
                                                   MaaSize index) { return (const MaaToolkitIOSDevice *)list; }
const char *MaaToolkitIOSDeviceGetName(const MaaToolkitIOSDevice *device) { return "fixture"; }
const char *MaaToolkitIOSDeviceGetUdid(const MaaToolkitIOSDevice *device) { return "offline-udid"; }
const char *MaaToolkitIOSDeviceGetState(const MaaToolkitIOSDevice *device) { return "discovered"; }
const char *MaaToolkitIOSDeviceGetConnections(const MaaToolkitIOSDevice *device) { return "[]"; }
MaaIOSScreencapMethod MaaToolkitIOSDeviceGetScreencapMethods(const MaaToolkitIOSDevice *device) { return 3; }
const char *MaaToolkitIOSDeviceListGetWarnings(const MaaToolkitIOSDeviceList *list) { return "[]"; }
#endif
