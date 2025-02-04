#pragma once

#include <stdint.h>

typedef void o3cview_viewer;

#define DISPLAY_WIDTH 160
#define DISPLAY_HEIGHT 80

#ifdef __cplusplus
extern "C" {
#endif

o3cview_viewer* o3cview_init();
void o3cview_free(o3cview_viewer* viewer);
void o3cview_get_frame(o3cview_viewer* viewer, uint8_t* fb);

#ifdef __cplusplus
}
#endif
