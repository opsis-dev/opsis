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
} OpsisUniform;

OpsisFilament* opsis_filament_new(int32_t backend);
void opsis_filament_free(OpsisFilament* filament);
int32_t opsis_filament_backend(const OpsisFilament* filament);
int32_t opsis_filament_material(OpsisFilament* filament, const uint8_t* bytes,
                                size_t size);
int32_t opsis_filament_pass(OpsisFilament* filament, int32_t material,
                            const uint8_t* const* inputs, size_t input_count,
                            uint32_t width, uint32_t height,
                            const OpsisUniform* uniforms, size_t uniform_count,
                            uint8_t* out);

#ifdef __cplusplus
}
#endif
