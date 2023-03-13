#ifndef OCTREE_GLSL
#define OCTREE_GLSL

#define MEMORY_SIZE 1024

struct Bounds {
    vec3 upper;
    vec3 lower;
};

struct Octree {
    Bounds bounds;
    uint descriptors[MEMORY_SIZE];
    uint free_address;
};

#endif