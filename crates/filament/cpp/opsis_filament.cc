#include "opsis_filament.h"

#include <backend/PixelBufferDescriptor.h>
#include <filament/Camera.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
#include <filament/Material.h>
#include <filament/MaterialInstance.h>
#include <filament/RenderableManager.h>
#include <filament/Renderer.h>
#include <filament/Scene.h>
#include <filament/SwapChain.h>
#include <filament/Texture.h>
#include <filament/TextureSampler.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Viewport.h>
#include <math/vec2.h>
#include <math/vec3.h>
#include <math/vec4.h>
#include <utils/EntityManager.h>

#include <cstring>
#include <memory>
#include <string>
#include <vector>

#include "opsis_internal.h"

using filament::Camera;
using filament::Engine;
using filament::IndexBuffer;
using filament::Material;
using filament::MaterialInstance;
using filament::RenderableManager;
using filament::Renderer;
using filament::Scene;
using filament::SwapChain;
using filament::Texture;
using filament::TextureSampler;
using filament::VertexAttribute;
using filament::VertexBuffer;
using filament::View;

namespace {

constexpr float kTriangle[] = {-1.0f, -1.0f, 0.0f,  1.0f, 3.0f, -1.0f,
                               2.0f,  1.0f,  -1.0f, 3.0f, 0.0f, -1.0f};
constexpr uint16_t kIndices[] = {0, 1, 2};
constexpr int32_t kFailed = -1;
constexpr uint32_t kVertexStride = 16;
constexpr uint32_t kUvOffset = 8;

Engine::Backend backend_of(int32_t backend) {
  switch (backend) {
    case 1:
      return Engine::Backend::OPENGL;
    case 2:
      return Engine::Backend::METAL;
    default:
      return Engine::Backend::VULKAN;
  }
}

int32_t backend_code(Engine::Backend backend) {
  switch (backend) {
    case Engine::Backend::OPENGL:
      return 1;
    case Engine::Backend::METAL:
      return 2;
    default:
      return 0;
  }
}

}

extern "C" OpsisFilament* opsis_filament_new(int32_t backend) {
  Engine* engine = Engine::Builder().backend(backend_of(backend)).build();
  if (engine == nullptr) {
    return nullptr;
  }
  auto* made = new OpsisFilament();
  made->engine = engine;
  made->renderer = engine->createRenderer();
  made->scene = engine->createScene();
  made->view = engine->createView();
  made->eye = utils::EntityManager::get().create();
  made->camera = engine->createCamera(made->eye);
  made->view->setCamera(made->camera);
  made->view->setScene(made->scene);
  made->view->setPostProcessingEnabled(false);
  made->vertices =
      VertexBuffer::Builder()
          .vertexCount(3)
          .bufferCount(1)
          .attribute(VertexAttribute::POSITION, 0,
                     VertexBuffer::AttributeType::FLOAT2, 0, kVertexStride)
          .attribute(VertexAttribute::UV0, 0,
                     VertexBuffer::AttributeType::FLOAT2, kUvOffset,
                     kVertexStride)
          .build(*engine);
  made->vertices->setBufferAt(
      *engine, 0, VertexBuffer::BufferDescriptor(kTriangle, sizeof(kTriangle)));
  made->indices = IndexBuffer::Builder()
                      .indexCount(3)
                      .bufferType(IndexBuffer::IndexType::USHORT)
                      .build(*engine);
  made->indices->setBuffer(
      *engine, IndexBuffer::BufferDescriptor(kIndices, sizeof(kIndices)));
  made->quad = utils::EntityManager::get().create();
  return made;
}

extern "C" void opsis_filament_free(OpsisFilament* filament) {
  if (filament == nullptr) {
    return;
  }
  Engine* engine = filament->engine;
  opsis::free_models(filament);
  engine->destroy(filament->quad);
  utils::EntityManager::get().destroy(filament->quad);
  for (Material* material : filament->materials) {
    engine->destroy(material);
  }
  if (filament->grading != nullptr) {
    engine->destroy(filament->grading);
  }
  engine->destroy(filament->indices);
  engine->destroy(filament->vertices);
  engine->destroyCameraComponent(filament->eye);
  utils::EntityManager::get().destroy(filament->eye);
  engine->destroy(filament->view);
  engine->destroy(filament->scene);
  engine->destroy(filament->renderer);
  Engine::destroy(&engine);
  delete filament;
}

extern "C" int32_t opsis_filament_backend(const OpsisFilament* filament) {
  return backend_code(filament->engine->getBackend());
}

extern "C" int32_t opsis_filament_material(OpsisFilament* filament,
                                           const uint8_t* bytes, size_t size) {
  Material* material =
      Material::Builder().package(bytes, size).build(*filament->engine);
  if (material == nullptr) {
    return kFailed;
  }
  filament->materials.push_back(material);
  return static_cast<int32_t>(filament->materials.size() - 1);
}

extern "C" int32_t opsis_filament_pass(
    OpsisFilament* filament, int32_t material, const OpsisInput* inputs,
    size_t input_count, uint32_t width, uint32_t height,
    const OpsisUniform* uniforms, size_t uniform_count, uint8_t* out) {
  if (material < 0 ||
      static_cast<size_t>(material) >= filament->materials.size()) {
    return kFailed;
  }
  Engine* engine = filament->engine;
  size_t size = static_cast<size_t>(width) * height * 4;
  Material* chosen = filament->materials[static_cast<size_t>(material)];
  MaterialInstance* instance = chosen->createInstance();
  std::vector<Texture*> textures;
  for (size_t i = 0; i < input_count; ++i) {
    Texture* texture = opsis::upload(engine, inputs[i].pixels, inputs[i].width,
                                     inputs[i].height);
    textures.push_back(texture);
    std::string name = "input" + std::to_string(i);
    if (!chosen->hasParameter(name.c_str())) {
      continue;
    }
    instance->setParameter(name.c_str(), texture,
                           TextureSampler(TextureSampler::MinFilter::LINEAR,
                                          TextureSampler::MagFilter::LINEAR));
  }
  bool shaped = true;
  for (size_t i = 0; i < uniform_count; ++i) {
    if (!chosen->hasParameter(uniforms[i].name)) {
      continue;
    }
    shaped = opsis::set_uniform(instance, uniforms[i]) && shaped;
  }
  if (!shaped) {
    engine->destroy(instance);
    for (Texture* texture : textures) {
      engine->destroy(texture);
    }
    return kFailed;
  }
  RenderableManager::Builder(1)
      .boundingBox({{-1, -1, -1}, {1, 1, 1}})
      .material(0, instance)
      .geometry(0, RenderableManager::PrimitiveType::TRIANGLES,
                filament->vertices, filament->indices)
      .culling(false)
      .build(*engine, filament->quad);
  filament->scene->addEntity(filament->quad);
  SwapChain* swap =
      engine->createSwapChain(width, height, SwapChain::CONFIG_READABLE);
  filament->view->setViewport({0, 0, width, height});
  while (!filament->renderer->beginFrame(swap)) {
  }
  filament->renderer->render(filament->view);
  filament->renderer->readPixels(
      0, 0, width, height,
      filament::backend::PixelBufferDescriptor(
          out, size, filament::backend::PixelDataFormat::RGBA,
          filament::backend::PixelDataType::UBYTE));
  filament->renderer->endFrame();
  engine->flushAndWait();
  filament->scene->remove(filament->quad);
  engine->getRenderableManager().destroy(filament->quad);
  engine->destroy(swap);
  engine->destroy(instance);
  for (Texture* texture : textures) {
    engine->destroy(texture);
  }
  return 0;
}

namespace opsis {

constexpr size_t kScalar = 1;
constexpr size_t kPair = 2;
constexpr size_t kTriple = 3;
constexpr size_t kQuad = 4;

Texture* upload(Engine* engine, const uint8_t* source, uint32_t width,
                uint32_t height) {
  size_t size = static_cast<size_t>(width) * height * 4;
  auto* pixels = new uint8_t[size];
  std::memcpy(pixels, source, size);
  Texture* texture = Texture::Builder()
                         .width(width)
                         .height(height)
                         .levels(1)
                         .format(Texture::InternalFormat::RGBA8)
                         .sampler(Texture::Sampler::SAMPLER_2D)
                         .build(*engine);
  texture->setImage(*engine, 0,
                    Texture::PixelBufferDescriptor(
                        pixels, size, Texture::Format::RGBA,
                        Texture::Type::UBYTE, [](void* buffer, size_t, void*) {
                          delete[] static_cast<uint8_t*>(buffer);
                        }));
  return texture;
}

bool set_array(MaterialInstance* instance, const OpsisUniform& uniform) {
  if (uniform.columns == 0 || uniform.count % uniform.columns != 0) {
    return false;
  }
  size_t length = uniform.count / uniform.columns;
  switch (uniform.columns) {
    case kScalar:
      instance->setParameter(uniform.name, uniform.values, length);
      return true;
    case kPair:
      instance->setParameter(
          uniform.name,
          reinterpret_cast<const filament::math::float2*>(uniform.values),
          length);
      return true;
    case kQuad:
      instance->setParameter(
          uniform.name,
          reinterpret_cast<const filament::math::float4*>(uniform.values),
          length);
      return true;
    default:
      return false;
  }
}

bool set_uniform(MaterialInstance* instance, const OpsisUniform& uniform) {
  if (uniform.count != uniform.columns) {
    return set_array(instance, uniform);
  }
  const float* v = uniform.values;
  switch (uniform.count) {
    case kScalar:
      instance->setParameter(uniform.name, v[0]);
      return true;
    case kPair:
      instance->setParameter(uniform.name, filament::math::float2{v[0], v[1]});
      return true;
    case kTriple:
      instance->setParameter(uniform.name,
                             filament::math::float3{v[0], v[1], v[2]});
      return true;
    case kQuad:
      instance->setParameter(uniform.name,
                             filament::math::float4{v[0], v[1], v[2], v[3]});
      return true;
    default:
      return false;
  }
}

}
