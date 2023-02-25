#ifndef CAMERA_GLSL
#define CAMERA_GLSL

struct Camera {
    vec3 origin;
    vec3 lower_left_corner;
    vec3 horizontal;
    vec3 vertical;
};

#endif