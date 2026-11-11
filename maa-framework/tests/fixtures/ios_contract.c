/* Device-free ABI fixture. No XDeviceBridge, iOS device, or SDK is loaded. */
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef uint8_t MaaBool;
typedef uint64_t MaaSize;
typedef uint64_t MaaIOSScreencapMethod;
typedef int32_t MaaIOSConnectionMethod;
typedef MaaBool (*MaaToolkitIOSDeviceCancelCallback)(void *);

typedef struct MaaController { int unused; } MaaController;
typedef struct MaaToolkitIOSDevice {
    char *name, *udid, *state, *connections;
} MaaToolkitIOSDevice;
typedef struct MaaToolkitIOSDeviceList {
    MaaToolkitIOSDevice device;
    char *warnings;
} MaaToolkitIOSDeviceList;

/* Counts: list create/destroy, find/options, controller create/destroy/clear,
 * callbacks, live lists, live controllers. */
static uint64_t counts[10];
static int scenario;
static int32_t last_connection;
static uint64_t last_methods;
static char last_udid[256], last_config[512], last_options[512];

void MockSetScenario(int value) { scenario = value; }
uint64_t MockCount(int index) { return counts[index]; }
int32_t MockLastConnection(void) { return last_connection; }
uint64_t MockLastMethods(void) { return last_methods; }
const char *MockLastUdid(void) { return last_udid; }
const char *MockLastConfig(void) { return last_config; }
const char *MockLastOptions(void) { return last_options; }

#ifndef OMIT_CONTROLLER_CREATE
MaaController *MaaIOSControllerCreate(const char *udid,
                                    MaaIOSScreencapMethod methods,
                                    MaaIOSConnectionMethod connection,
                                    const char *config)
{
    ++counts[4];
    last_connection = connection;
    last_methods = methods;
    snprintf(last_udid, sizeof(last_udid), "%s", udid);
    snprintf(last_config, sizeof(last_config), "%s", config);
    if (scenario == 1) return NULL;
    MaaController *controller = calloc(1, sizeof(*controller));
    if (controller) ++counts[9];
    return controller;
}
#endif

#ifndef OMIT_CONTROLLER_DESTROY
void MaaControllerDestroy(MaaController *controller)
{
    ++counts[5];
    --counts[9];
    free(controller);
}
#endif

#ifndef OMIT_CONTROLLER_CLEAR
void MaaControllerClearSinks(MaaController *controller)
{
    if (!controller) abort();
    ++counts[6];
}
#endif

MaaToolkitIOSDeviceList *MaaToolkitIOSDeviceListCreate(void)
{
    ++counts[0];
    if (scenario == 2) return NULL;
    MaaToolkitIOSDeviceList *list = calloc(1, sizeof(*list));
    if (!list) return NULL;
    list->device.name = strdup("Test iPhone");
    list->device.udid = strdup("offline-udid");
    list->device.state = strdup("discovered");
    list->device.connections = strdup(scenario == 6 ? "invalid" :
        scenario == 8 ? "{}" : "[{\"transport\":\"usb\",\"paired\":false,\"port\":49152}]");
    list->warnings = strdup(scenario == 7 ? "invalid" :
        scenario == 9 ? "{}" : "[\"offline fixture\"]");
    ++counts[8];
    return list;
}

#ifndef OMIT_LIST_DESTROY
void MaaToolkitIOSDeviceListDestroy(MaaToolkitIOSDeviceList *list)
{
    if (!list) abort();
    ++counts[1];
    --counts[8];
    /* Poison borrowed data before freeing: Rust results must own their strings. */
    memset(list->device.name, 'x', strlen(list->device.name));
    memset(list->device.udid, 'x', strlen(list->device.udid));
    memset(list->device.state, 'x', strlen(list->device.state));
    memset(list->device.connections, 'x', strlen(list->device.connections));
    memset(list->warnings, 'x', strlen(list->warnings));
    free(list->device.name);
    free(list->device.udid);
    free(list->device.state);
    free(list->device.connections);
    free(list->warnings);
    free(list);
}
#endif

MaaBool MaaToolkitIOSDeviceFind(MaaToolkitIOSDeviceList *list)
{
    if (!list) abort();
    ++counts[2];
    return scenario != 3;
}

struct CallbackArgs {
    MaaToolkitIOSDeviceCancelCallback callback;
    void *userdata;
    MaaBool cancelled;
};

static void *run_callback(void *raw)
{
    struct CallbackArgs *args = raw;
    ++counts[7];
    args->cancelled = args->callback(args->userdata);
    return NULL;
}

#ifndef OMIT_FIND_OPTIONS
MaaBool MaaToolkitIOSDeviceFindWithOptions(
    MaaIOSConnectionMethod connection, const char *options,
    MaaToolkitIOSDeviceCancelCallback callback, void *userdata,
    MaaToolkitIOSDeviceList *list)
{
    if (!list) abort();
    ++counts[3];
    last_connection = connection;
    snprintf(last_options, sizeof(last_options), "%s", options);
    if (callback) {
        struct CallbackArgs args = {callback, userdata, 0};
        pthread_t worker;
        if (pthread_create(&worker, NULL, run_callback, &args) != 0) abort();
        if (pthread_join(worker, NULL) != 0) abort();
        if (args.cancelled) {
            free(list->warnings);
            list->warnings = strdup("[\"cancelled\"]");
            return 0;
        }
    } else if (userdata) {
        abort();
    }
    return scenario != 3;
}
#endif

MaaSize MaaToolkitIOSDeviceListSize(const MaaToolkitIOSDeviceList *list)
{
    if (!list) abort();
    return scenario == 10 ? 0 : 1;
}

const MaaToolkitIOSDevice *MaaToolkitIOSDeviceListAt(
    const MaaToolkitIOSDeviceList *list, MaaSize index)
{
    if (!list || index != 0) abort();
    return scenario == 4 ? NULL : &list->device;
}

const char *MaaToolkitIOSDeviceGetName(const MaaToolkitIOSDevice *device)
{
    if (!device) abort();
    return scenario == 5 ? NULL : device->name;
}
const char *MaaToolkitIOSDeviceGetUdid(const MaaToolkitIOSDevice *device)
{
    return scenario == 11 ? NULL : device->udid;
}
#ifndef OMIT_GET_STATE
const char *MaaToolkitIOSDeviceGetState(const MaaToolkitIOSDevice *device)
{
    return scenario == 12 ? NULL : device->state;
}
#endif
const char *MaaToolkitIOSDeviceGetConnections(const MaaToolkitIOSDevice *device)
{
    return scenario == 13 ? NULL : device->connections;
}
MaaIOSScreencapMethod MaaToolkitIOSDeviceGetScreencapMethods(
    const MaaToolkitIOSDevice *device)
{
    if (!device) abort();
    /* Includes an unknown future flag to ensure it survives discovery. */
    return UINT64_C(0x8000000000000003);
}
const char *MaaToolkitIOSDeviceListGetWarnings(const MaaToolkitIOSDeviceList *list)
{
    return scenario == 14 ? NULL : list->warnings;
}
