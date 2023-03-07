#ifndef RAY_GLSL
#define RAY_GLSL

#include "camera.glsl"

struct Ray {
    vec3 origin;
    vec3 direction;
};

Ray create_ray(Camera camera, float u, float v) {
    Ray ray = Ray(
        camera.origin, 
        camera.upper_left_corner + u * camera.horizontal - v * camera.vertical - camera.origin
    );

    return ray;
}

#endif