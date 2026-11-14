#pragma once

#include <filament/Camera.h>
#include <filament/ColorGrading.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
#include <filament/Material.h>
#include <filament/MaterialInstance.h>
#include <filament/Renderer.h>
#include <filament/Scene.h>
#include <filament/Texture.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <utils/Entity.h>

#include <vector>

#include "opsis_filament.h"

struct OpsisFilament {
  filament::Engine* engine = nullptr;
  filament::Renderer* renderer = nullptr;
  filament::Scene* scene = nullptr;
  filament::View* view = nullptr;
  filament::Camera* camera = nullptr;
  utils::Entity eye;
  utils::Entity quad;
  filament::VertexBuffer* vertices = nullptr;
  filament::IndexBuffer* indices = nullptr;
  std::vector<filament::Material*> materials;
  filament::ColorGrading* grading = nullptr;
};

namespace opsis {

filament::Texture* upload(filament::Engine* engine, const uint8_t* source,
                          uint32_t width, uint32_t height);
bool set_uniform(filament::MaterialInstance* instance,
                 const OpsisUniform& uniform);

}
