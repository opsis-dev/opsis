#include <backend/PixelBufferDescriptor.h>
#include <filament/Box.h>
#include <filament/Camera.h>
#include <filament/Color.h>
#include <filament/ColorGrading.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
#include <filament/IndirectLight.h>
#include <filament/LightManager.h>
#include <filament/Material.h>
#include <filament/MaterialInstance.h>
#include <filament/Options.h>
#include <filament/RenderableManager.h>
#include <filament/Renderer.h>
#include <filament/Scene.h>
#include <filament/SwapChain.h>
#include <filament/Texture.h>
#include <filament/TextureSampler.h>
#include <filament/ToneMapper.h>
#include <filament/TransformManager.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Viewport.h>
#include <geometry/SurfaceOrientation.h>
#include <image/Ktx1Bundle.h>
#include <ktxreader/Ktx1Reader.h>
#include <math/mat4.h>
#include <math/vec3.h>
#include <math/vec4.h>
#include <utils/EntityManager.h>

#include <algorithm>
#include <cmath>
#include <cstring>
#include <limits>
#include <string>
#include <vector>

#include "opsis_filament.h"
#include "opsis_internal.h"

using filament::Engine;
using filament::IndexBuffer;
using filament::Material;
using filament::MaterialInstance;
using filament::RenderableManager;
using filament::Texture;
using filament::TextureSampler;
using filament::VertexAttribute;
using filament::VertexBuffer;
using filament::math::float3;
using filament::math::float4;
using filament::math::mat4f;
using filament::math::short4;

namespace {

constexpr int32_t kSceneFailed = -1;
constexpr float kExposure = 1.0f;
constexpr float kAcesScale = 1.0f / 0.6f;

float3 times(const float m[3][3], float3 c) {
  return {m[0][0] * c.x + m[0][1] * c.y + m[0][2] * c.z,
          m[1][0] * c.x + m[1][1] * c.y + m[1][2] * c.z,
          m[2][0] * c.x + m[2][1] * c.y + m[2][2] * c.z};
}

constexpr float kRec2020To709[3][3] = {{1.6604910f, -0.5876411f, -0.0728499f},
                                       {-0.1245505f, 1.1328999f, -0.0083494f},
                                       {-0.0181508f, -0.1005789f, 1.1187297f}};
constexpr float kRec709To2020[3][3] = {{0.6274040f, 0.3292820f, 0.0433136f},
                                       {0.0690970f, 0.9195400f, 0.0113612f},
                                       {0.0163916f, 0.0880132f, 0.8955950f}};
constexpr float kAcesIn[3][3] = {{0.59719f, 0.35458f, 0.04823f},
                                 {0.07600f, 0.90834f, 0.01566f},
                                 {0.02840f, 0.13383f, 0.83777f}};
constexpr float kAcesOut[3][3] = {{1.60475f, -0.53108f, -0.07367f},
                                  {-0.10208f, 1.10813f, -0.00605f},
                                  {-0.00327f, -0.07276f, 1.07602f}};

float fit(float v) {
  float a = v * (v + 0.0245786f) - 0.000090537f;
  float b = v * (0.983729f * v + 0.4329510f) + 0.238081f;
  return a / b;
}

struct ThreeAcesToneMapper final : public filament::ToneMapper {
  float3 operator()(float3 c) const noexcept override {
    float3 rgb = times(kRec2020To709, c) * kAcesScale;
    rgb = times(kAcesIn, rgb);
    rgb = {fit(rgb.x), fit(rgb.y), fit(rgb.z)};
    rgb = times(kAcesOut, rgb);
    rgb = {std::clamp(rgb.x, 0.0f, 1.0f), std::clamp(rgb.y, 0.0f, 1.0f),
           std::clamp(rgb.z, 0.0f, 1.0f)};
    return times(kRec709To2020, rgb);
  }
};

filament::ColorGrading* grading_of(OpsisFilament* filament) {
  if (filament->grading == nullptr) {
    ThreeAcesToneMapper mapper;
    filament->grading =
        filament::ColorGrading::Builder()
            .quality(filament::ColorGrading::QualityLevel::ULTRA)
            .toneMapper(&mapper)
            .build(*filament->engine);
  }
  return filament->grading;
}

template <typename T>
VertexBuffer::BufferDescriptor copied(const T* data, size_t count) {
  size_t bytes = count * sizeof(T);
  auto* copy = new uint8_t[bytes];
  std::memcpy(copy, data, bytes);
  return VertexBuffer::BufferDescriptor(
      copy, bytes, [](void* buffer, size_t, void*) {
        delete[] static_cast<uint8_t*>(buffer);
      });
}

Texture* image_texture(Engine* engine, const OpsisInput& input) {
  uint32_t levels = 1;
  for (uint32_t side = std::max(input.width, input.height); side > 1;
       side /= 2) {
    ++levels;
  }
  size_t size = static_cast<size_t>(input.width) * input.height * 4;
  auto* pixels = new uint8_t[size];
  std::memcpy(pixels, input.pixels, size);
  Texture* texture =
      Texture::Builder()
          .width(input.width)
          .height(input.height)
          .levels(levels)
          .format(Texture::InternalFormat::SRGB8_A8)
          .sampler(Texture::Sampler::SAMPLER_2D)
          .usage(Texture::Usage::DEFAULT | Texture::Usage::GEN_MIPMAPPABLE)
          .build(*engine);
  texture->setImage(*engine, 0,
                    Texture::PixelBufferDescriptor(
                        pixels, size, Texture::Format::RGBA,
                        Texture::Type::UBYTE, [](void* buffer, size_t, void*) {
                          delete[] static_cast<uint8_t*>(buffer);
                        }));
  texture->generateMipmaps(*engine);
  return texture;
}

struct Built {
  utils::Entity entity;
  VertexBuffer* vertices = nullptr;
  IndexBuffer* indices = nullptr;
  MaterialInstance* instance = nullptr;
  std::vector<Texture*> textures;
};

bool build_mesh(OpsisFilament* filament, const OpsisMesh& mesh, Built& built) {
  if (mesh.material < 0 ||
      static_cast<size_t>(mesh.material) >= filament->materials.size() ||
      mesh.vertex_count == 0 || mesh.index_count < 3) {
    return false;
  }
  Engine* engine = filament->engine;
  const auto* positions = reinterpret_cast<const float3*>(mesh.positions);
  const auto* normals = reinterpret_cast<const float3*>(mesh.normals);
  std::vector<short4> quats(mesh.vertex_count);
  auto* orientation = filament::geometry::SurfaceOrientation::Builder()
                          .vertexCount(mesh.vertex_count)
                          .normals(normals)
                          .build();
  orientation->getQuats(quats.data(), mesh.vertex_count);
  delete orientation;
  bool textured = mesh.uvs != nullptr;
  auto builder = VertexBuffer::Builder()
                     .vertexCount(mesh.vertex_count)
                     .bufferCount(textured ? 3 : 2)
                     .attribute(VertexAttribute::POSITION, 0,
                                VertexBuffer::AttributeType::FLOAT3)
                     .attribute(VertexAttribute::TANGENTS, 1,
                                VertexBuffer::AttributeType::SHORT4)
                     .normalized(VertexAttribute::TANGENTS);
  if (textured) {
    builder.attribute(VertexAttribute::UV0, 2,
                      VertexBuffer::AttributeType::FLOAT2);
  }
  built.vertices = builder.build(*engine);
  built.vertices->setBufferAt(*engine, 0, copied(positions, mesh.vertex_count));
  built.vertices->setBufferAt(*engine, 1, copied(quats.data(), quats.size()));
  if (textured) {
    built.vertices->setBufferAt(
        *engine, 2,
        copied(reinterpret_cast<const filament::math::float2*>(mesh.uvs),
               mesh.vertex_count));
  }
  built.indices = IndexBuffer::Builder()
                      .indexCount(mesh.index_count)
                      .bufferType(IndexBuffer::IndexType::UINT)
                      .build(*engine);
  {
    size_t bytes = static_cast<size_t>(mesh.index_count) * sizeof(uint32_t);
    auto* copy = new uint8_t[bytes];
    std::memcpy(copy, mesh.indices, bytes);
    built.indices->setBuffer(*engine,
                             IndexBuffer::BufferDescriptor(
                                 copy, bytes, [](void* buffer, size_t, void*) {
                                   delete[] static_cast<uint8_t*>(buffer);
                                 }));
  }
  Material* material = filament->materials[static_cast<size_t>(mesh.material)];
  built.instance = material->createInstance();
  for (size_t i = 0; i < mesh.uniform_count; ++i) {
    if (material->hasParameter(mesh.uniforms[i].name)) {
      opsis::set_uniform(built.instance, mesh.uniforms[i]);
    }
  }
  for (size_t i = 0; i < mesh.texture_count; ++i) {
    std::string name = "texture" + std::to_string(i);
    if (!material->hasParameter(name.c_str())) {
      continue;
    }
    Texture* texture = image_texture(engine, mesh.textures[i]);
    built.textures.push_back(texture);
    built.instance->setParameter(
        name.c_str(), texture,
        TextureSampler(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR,
                       TextureSampler::MagFilter::LINEAR));
  }
  float3 low{std::numeric_limits<float>::max()};
  float3 high{std::numeric_limits<float>::lowest()};
  for (uint32_t i = 0; i < mesh.vertex_count; ++i) {
    low = min(low, positions[i]);
    high = max(high, positions[i]);
  }
  built.entity = utils::EntityManager::get().create();
  RenderableManager::Builder(1)
      .boundingBox({low, high})
      .material(0, built.instance)
      .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, built.vertices,
                built.indices)
      .culling(false)
      .castShadows(false)
      .receiveShadows(false)
      .build(*engine, built.entity);
  auto& transforms = engine->getTransformManager();
  transforms.create(built.entity);
  mat4f matrix;
  std::memcpy(&matrix, mesh.transform, sizeof(mesh.transform));
  transforms.setTransform(transforms.getInstance(built.entity), matrix);
  return true;
}

void destroy_built(Engine* engine, Built& built) {
  engine->getRenderableManager().destroy(built.entity);
  engine->getTransformManager().destroy(built.entity);
  utils::EntityManager::get().destroy(built.entity);
  if (built.instance != nullptr) engine->destroy(built.instance);
  if (built.indices != nullptr) engine->destroy(built.indices);
  if (built.vertices != nullptr) engine->destroy(built.vertices);
  for (Texture* texture : built.textures) engine->destroy(texture);
}

}

extern "C" int32_t opsis_filament_scene(OpsisFilament* filament,
                                        const OpsisScene* scene, uint32_t width,
                                        uint32_t height, uint8_t* out) {
  Engine* engine = filament->engine;
  filament::Scene* world = engine->createScene();
  filament::View* view = engine->createView();
  utils::Entity eye = utils::EntityManager::get().create();
  filament::Camera* camera = engine->createCamera(eye);
  camera->setProjection(
      scene->fov_degrees,
      static_cast<double>(width) / static_cast<double>(height),
      scene->near_plane, scene->far_plane, filament::Camera::Fov::VERTICAL);
  camera->lookAt({scene->eye[0], scene->eye[1], scene->eye[2]},
                 {scene->target[0], scene->target[1], scene->target[2]},
                 {scene->up[0], scene->up[1], scene->up[2]});
  camera->setExposure(kExposure);
  view->setCamera(camera);
  view->setScene(world);
  view->setViewport({0, 0, width, height});
  view->setPostProcessingEnabled(true);
  view->setBlendMode(filament::View::BlendMode::TRANSLUCENT);
  view->setColorGrading(grading_of(filament));
  view->setAntiAliasing(filament::View::AntiAliasing::NONE);
  view->setDithering(filament::View::Dithering::NONE);
  view->setShadowingEnabled(false);
  filament::MultiSampleAntiAliasingOptions msaa;
  msaa.enabled = scene->samples > 1;
  msaa.sampleCount =
      static_cast<uint8_t>(std::max<uint32_t>(scene->samples, 1));
  view->setMultiSampleAntiAliasingOptions(msaa);

  std::vector<Built> meshes(scene->mesh_count);
  bool ok = true;
  for (size_t i = 0; i < scene->mesh_count; ++i) {
    ok = build_mesh(filament, scene->meshes[i], meshes[i]) && ok;
    if (meshes[i].instance != nullptr) world->addEntity(meshes[i].entity);
  }
  std::vector<utils::Entity> lights;
  for (size_t i = 0; i < scene->light_count; ++i) {
    const OpsisLight& light = scene->lights[i];
    utils::Entity entity = utils::EntityManager::get().create();
    filament::LightManager::Builder(filament::LightManager::Type::DIRECTIONAL)
        .color({light.color[0], light.color[1], light.color[2]})
        .intensity(light.intensity)
        .direction({light.direction[0], light.direction[1], light.direction[2]})
        .castShadows(false)
        .build(*engine, entity);
    world->addEntity(entity);
    lights.push_back(entity);
  }
  Texture* reflections = nullptr;
  filament::IndirectLight* indirect = nullptr;
  if (scene->reflections != nullptr && scene->reflections_size > 0) {
    image::Ktx1Bundle bundle(scene->reflections,
                             static_cast<uint32_t>(scene->reflections_size));
    reflections = ktxreader::Ktx1Reader::createTexture(engine, bundle, false,
                                                       nullptr, nullptr);
  }
  if (reflections != nullptr || scene->irradiance_bands > 0) {
    auto builder = filament::IndirectLight::Builder().intensity(
        scene->environment_intensity);
    if (reflections != nullptr) builder.reflections(reflections);
    if (scene->irradiance_bands > 0) {
      builder.irradiance(static_cast<uint8_t>(scene->irradiance_bands),
                         reinterpret_cast<const float3*>(scene->irradiance));
    }
    indirect = builder.build(*engine);
    world->setIndirectLight(indirect);
  }

  int32_t result = kSceneFailed;
  if (ok) {
    filament::Renderer::ClearOptions clear;
    clear.clearColor = {0.0, 0.0, 0.0, 0.0};
    clear.clear = true;
    clear.discard = true;
    filament->renderer->setClearOptions(clear);
    filament::SwapChain* swap = engine->createSwapChain(
        width, height, filament::SwapChain::CONFIG_READABLE);
    while (!filament->renderer->beginFrame(swap)) {
    }
    filament->renderer->render(view);
    size_t size = static_cast<size_t>(width) * height * 4;
    filament->renderer->readPixels(
        0, 0, width, height,
        filament::backend::PixelBufferDescriptor(
            out, size, filament::backend::PixelDataFormat::RGBA,
            filament::backend::PixelDataType::UBYTE));
    filament->renderer->endFrame();
    engine->flushAndWait();
    engine->destroy(swap);
    result = 0;
  }

  for (Built& built : meshes) destroy_built(engine, built);
  for (utils::Entity entity : lights) {
    engine->getLightManager().destroy(entity);
    utils::EntityManager::get().destroy(entity);
  }
  if (indirect != nullptr) engine->destroy(indirect);
  if (reflections != nullptr) engine->destroy(reflections);
  engine->destroyCameraComponent(eye);
  utils::EntityManager::get().destroy(eye);
  engine->destroy(view);
  engine->destroy(world);
  return result;
}
