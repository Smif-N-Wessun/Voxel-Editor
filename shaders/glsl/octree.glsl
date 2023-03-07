#ifndef OCTREE_GLSL
#define OCTREE_GLSL

#define MEMORY_SIZE 1024

struct Bounds {
    vec3 min;
    vec3 max;
};

struct Octree {
    Bounds bound;
    uint descriptors[1024];
};

#endif