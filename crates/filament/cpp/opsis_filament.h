#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct OpsisFilament OpsisFilament;

typedef struct OpsisUniform {
  const char* name;
  const float* values;
  size_t count;
  size_t columns;
} OpsisUniform;

typedef struct OpsisInput {
  const uint8_t* pixels;
  uint32_t width;
  uint32_t height;
} OpsisInput;

typedef struct OpsisMesh {
  const float* positions;
  const float* normals;
  const float* uvs;
  uint32_t vertex_count;
  const uint32_t* indices;
  uint32_t index_count;
  float transform[16];
  int32_t material;
  const OpsisUniform* uniforms;
  size_t uniform_count;
  const OpsisInput* textures;
  size_t texture_count;
} OpsisMesh;

typedef struct OpsisLight {
  float direction[3];
  float color[3];
  float intensity;
} OpsisLight;

typedef struct OpsisModelUse {
  int32_t model;
  float transform[16];
  int32_t animation;
  float time;
} OpsisModelUse;

typedef struct OpsisScene {
  float fov_degrees;
  float near_plane;
  float far_plane;
  float eye[3];
  float target[3];
  float up[3];
  const OpsisMesh* meshes;
  size_t mesh_count;
  const OpsisLight* lights;
  size_t light_count;
  const OpsisModelUse* models;
  size_t model_count;
  const uint8_t* reflections;
  size_t reflections_size;
  const float* irradiance;
  uint32_t irradiance_bands;
  float environment_intensity;
  uint32_t samples;
} OpsisScene;

OpsisFilament* opsis_filament_new(int32_t backend);
void opsis_filament_free(OpsisFilament* filament);
int32_t opsis_filament_backend(const OpsisFilament* filament);
int32_t opsis_filament_material(OpsisFilament* filament, const uint8_t* bytes,
                                size_t size);
int32_t opsis_filament_pass(OpsisFilament* filament, int32_t material,
                            const OpsisInput* inputs, size_t input_count,
                            uint32_t width, uint32_t height,
                            const OpsisUniform* uniforms, size_t uniform_count,
                            uint8_t* out);
int32_t opsis_filament_model(OpsisFilament* filament, const uint8_t* bytes,
                             size_t size);
uint32_t opsis_filament_model_animations(OpsisFilament* filament,
                                         int32_t model);
const char* opsis_filament_model_animation(OpsisFilament* filament,
                                           int32_t model, uint32_t index,
                                           float* duration);
int32_t opsis_filament_model_bounds(OpsisFilament* filament, int32_t model,
                                    int32_t animation, float time, float* out);
int32_t opsis_filament_scene(OpsisFilament* filament, const OpsisScene* scene,
                             uint32_t width, uint32_t height, uint8_t* out);

#ifdef __cplusplus
}
#endif
