#include <filament/Box.h>
#include <filament/Engine.h>
#include <filament/RenderableManager.h>
#include <filament/TransformManager.h>
#include <gltfio/Animator.h>
#include <gltfio/AssetLoader.h>
#include <gltfio/FilamentAsset.h>
#include <gltfio/FilamentInstance.h>
#include <gltfio/MaterialProvider.h>
#include <gltfio/ResourceLoader.h>
#include <gltfio/TextureProvider.h>
#include <gltfio/materials/uberarchive.h>
#include <math/mat4.h>
#include <math/vec3.h>
#include <math/vec4.h>

#include <algorithm>
#include <cstring>
#include <limits>

#include "opsis_filament.h"
#include "opsis_internal.h"

using filament::Engine;
using filament::gltfio::FilamentAsset;
using filament::math::float3;
using filament::math::float4;
using filament::math::mat4f;

namespace {

constexpr int32_t kModelFailed = -1;

void ensure_loader(OpsisFilament* filament) {
  if (filament->model_loader != nullptr) {
    return;
  }
  Engine* engine = filament->engine;
  filament->model_materials = filament::gltfio::createUbershaderProvider(
      engine, UBERARCHIVE_DEFAULT_DATA, UBERARCHIVE_DEFAULT_SIZE);
  filament->model_loader = filament::gltfio::AssetLoader::create(
      {engine, filament->model_materials});
  filament->model_resources = new filament::gltfio::ResourceLoader(
      {.engine = engine, .normalizeSkinningWeights = true});
  filament->stb_textures = filament::gltfio::createStbProvider(engine);
  filament->ktx2_textures = filament::gltfio::createKtx2Provider(engine);
  filament->model_resources->addTextureProvider("image/png",
                                                filament->stb_textures);
  filament->model_resources->addTextureProvider("image/jpeg",
                                                filament->stb_textures);
  filament->model_resources->addTextureProvider("image/ktx2",
                                                filament->ktx2_textures);
}

filament::gltfio::Animator* animator_of(FilamentAsset* asset) {
  filament::gltfio::FilamentInstance* instance = asset->getInstance();
  return instance == nullptr ? nullptr : instance->getAnimator();
}

}

namespace opsis {

FilamentAsset* model_of(OpsisFilament* filament, int32_t model) {
  if (model < 0 || static_cast<size_t>(model) >= filament->models.size()) {
    return nullptr;
  }
  return filament->models[static_cast<size_t>(model)];
}

void pose_model(OpsisFilament* filament, FilamentAsset* asset,
                const float* transform, int32_t animation, float time) {
  auto& transforms = filament->engine->getTransformManager();
  mat4f matrix;
  std::memcpy(&matrix, transform, sizeof(float) * 16);
  transforms.setTransform(transforms.getInstance(asset->getRoot()), matrix);
  filament::gltfio::Animator* animator = animator_of(asset);
  if (animator == nullptr) {
    return;
  }
  animator->resetBoneMatrices();
  if (animation >= 0 &&
      static_cast<size_t>(animation) < animator->getAnimationCount()) {
    animator->applyAnimation(static_cast<size_t>(animation), time);
  }
  animator->updateBoneMatrices();
}

void free_models(OpsisFilament* filament) {
  if (filament->model_loader == nullptr) {
    return;
  }
  for (FilamentAsset* asset : filament->models) {
    filament->model_loader->destroyAsset(asset);
  }
  filament->models.clear();
  delete filament->model_resources;
  delete filament->stb_textures;
  delete filament->ktx2_textures;
  filament->model_materials->destroyMaterials();
  delete filament->model_materials;
  filament::gltfio::AssetLoader::destroy(&filament->model_loader);
}

}

extern "C" int32_t opsis_filament_model(OpsisFilament* filament,
                                        const uint8_t* bytes, size_t size) {
  ensure_loader(filament);
  FilamentAsset* asset =
      filament->model_loader->createAsset(bytes, static_cast<uint32_t>(size));
  if (asset == nullptr) {
    return kModelFailed;
  }
  if (!filament->model_resources->loadResources(asset)) {
    filament->model_loader->destroyAsset(asset);
    return kModelFailed;
  }
  animator_of(asset);
  asset->releaseSourceData();
  filament->models.push_back(asset);
  return static_cast<int32_t>(filament->models.size() - 1);
}

extern "C" uint32_t opsis_filament_model_animations(OpsisFilament* filament,
                                                    int32_t model) {
  FilamentAsset* asset = opsis::model_of(filament, model);
  filament::gltfio::Animator* animator =
      asset == nullptr ? nullptr : animator_of(asset);
  return animator == nullptr
             ? 0
             : static_cast<uint32_t>(animator->getAnimationCount());
}

extern "C" const char* opsis_filament_model_animation(OpsisFilament* filament,
                                                      int32_t model,
                                                      uint32_t index,
                                                      float* duration) {
  FilamentAsset* asset = opsis::model_of(filament, model);
  filament::gltfio::Animator* animator =
      asset == nullptr ? nullptr : animator_of(asset);
  if (animator == nullptr || index >= animator->getAnimationCount()) {
    return nullptr;
  }
  *duration = animator->getAnimationDuration(index);
  const char* name = animator->getAnimationName(index);
  return name == nullptr ? "" : name;
}

extern "C" int32_t opsis_filament_model_bounds(OpsisFilament* filament,
                                               int32_t model, int32_t animation,
                                               float time, float* out) {
  FilamentAsset* asset = opsis::model_of(filament, model);
  if (asset == nullptr) {
    return kModelFailed;
  }
  const float identity[16] = {1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1};
  opsis::pose_model(filament, asset, identity, animation, time);
  auto& renderables = filament->engine->getRenderableManager();
  auto& transforms = filament->engine->getTransformManager();
  float3 low{std::numeric_limits<float>::max()};
  float3 high{std::numeric_limits<float>::lowest()};
  bool any = false;
  const utils::Entity* entities = asset->getRenderableEntities();
  for (size_t i = 0; i < asset->getRenderableEntityCount(); ++i) {
    auto renderable = renderables.getInstance(entities[i]);
    auto placed = transforms.getInstance(entities[i]);
    if (!renderable.isValid() || !placed.isValid()) {
      continue;
    }
    filament::Box box = renderables.getAxisAlignedBoundingBox(renderable);
    mat4f world = transforms.getWorldTransform(placed);
    for (int corner = 0; corner < 8; ++corner) {
      float3 sign{(corner & 1) ? 1.0f : -1.0f, (corner & 2) ? 1.0f : -1.0f,
                  (corner & 4) ? 1.0f : -1.0f};
      float3 local = box.center + box.halfExtent * sign;
      float4 p = world * float4(local, 1.0f);
      float3 at = p.xyz / p.w;
      low = min(low, at);
      high = max(high, at);
      any = true;
    }
  }
  if (!any) {
    return kModelFailed;
  }
  out[0] = low.x;
  out[1] = low.y;
  out[2] = low.z;
  out[3] = high.x;
  out[4] = high.y;
  out[5] = high.z;
  return 0;
}
