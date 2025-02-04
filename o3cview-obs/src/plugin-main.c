/*
o3cview-obs
Copyright (C) 2025 Khangaroo

This program is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation; either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License along
with this program. If not, see <https://www.gnu.org/licenses/>
*/

#include <obs-module.h>
#include <plugin-support.h>
#include "o3cview.h"

OBS_DECLARE_MODULE()

// TODO: Support multiple sources at the same time
// The O3C doesn't like multiple things sending requests at the same time,
// so I have to grab frames on a separate thread instead.
// That would require some reference counting work though...

static bool already_created = false;

typedef struct {
    o3cview_viewer* viewer;
    obs_source_t* source;
    gs_texture_t* texture;
} o3cview_source;

static const char* o3cview_get_name(void* type_data) {
    UNUSED_PARAMETER(type_data);
    return "o3cview";
}

static void* o3cview_create(obs_data_t* settings, obs_source_t* source) {
    UNUSED_PARAMETER(settings);

    if (already_created) {
        obs_log(LOG_ERROR, "Creating multiple sources is not supported");
        return NULL;
    }
    already_created = true;

    obs_enter_graphics();

    o3cview_source* o3c_source = bzalloc(sizeof(o3cview_source));
    o3c_source->viewer = o3cview_init();
    o3c_source->source = source;
    o3c_source->texture = gs_texture_create(DISPLAY_WIDTH, DISPLAY_HEIGHT, GS_BGRX, 1, NULL, GS_DYNAMIC);

    if (!o3c_source->viewer || !o3c_source->texture) {
        obs_log(LOG_ERROR, "Failed to initialize o3cview");
        if (o3c_source->viewer)
            o3cview_free(o3c_source->viewer);
        if (o3c_source->texture)
            gs_texture_destroy(o3c_source->texture);
        bfree(source);

        obs_leave_graphics();
        return NULL;
    }

    obs_leave_graphics();
    return o3c_source;
}

static void o3cview_destroy(void* data) {
    o3cview_source* source = data;
    o3cview_free(source->viewer);

    obs_enter_graphics();
    gs_texture_destroy(source->texture);
    obs_leave_graphics();

    bfree(source);
    already_created = false;
}

static uint32_t o3cview_get_width(void* data) {
    UNUSED_PARAMETER(data);
    return DISPLAY_WIDTH;
}

static uint32_t o3cview_get_height(void* data) {
    UNUSED_PARAMETER(data);
    return DISPLAY_HEIGHT;
}

static void o3cview_render(void* data, gs_effect_t* effect) {
    UNUSED_PARAMETER(effect);

    o3cview_source* source = data;
    static uint16_t fb565[DISPLAY_WIDTH * DISPLAY_HEIGHT] = {};
    static uint8_t fb888[DISPLAY_WIDTH * DISPLAY_HEIGHT * 4] = {};
    o3cview_get_frame(source->viewer, (uint8_t*)fb565);

    // Possibly overkill and pure shifts might look close enough
    // https://stackoverflow.com/questions/2442576/how-does-one-convert-16-bit-rgb565-to-24-bit-rgb888
    for (size_t i = 0; i < DISPLAY_WIDTH * DISPLAY_HEIGHT; i++) {
        uint32_t rgb565 = fb565[i];
        fb888[i * 4] = ((rgb565 & 0x001F) * 527 + 23) >> 6;
        fb888[i * 4 + 1] = (((rgb565 & 0x07E0) >> 5) * 259 + 33) >> 6;
        fb888[i * 4 + 2] = (((rgb565 & 0xF800) >> 11) * 527 + 23) >> 6;
    }

    obs_enter_graphics();
    gs_texture_set_image(source->texture, fb888, DISPLAY_WIDTH * 4, false);
    obs_source_draw(source->texture, 0, 0, 0, 0, false);
    obs_leave_graphics();
}

static struct obs_source_info o3cview_obs_info = {
    .id = "o3cview-obs",
    .type = OBS_SOURCE_TYPE_INPUT,
    .output_flags = OBS_SOURCE_VIDEO | OBS_SOURCE_DO_NOT_DUPLICATE,
    .get_name = o3cview_get_name,
    .create = o3cview_create,
    .destroy = o3cview_destroy,
    .get_width = o3cview_get_width,
    .get_height = o3cview_get_height,
    .video_render = o3cview_render,
};

bool obs_module_load() {
    obs_register_source(&o3cview_obs_info);
    return true;
}
