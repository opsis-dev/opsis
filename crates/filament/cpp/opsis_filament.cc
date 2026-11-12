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
#include <utils/EntityManager.h>

#include <cstring>
#include <memory>
#include <string>
#include <vector>

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

struct OpsisFilament {
  Engine* engine = nullptr;
  Renderer* renderer = nullptr;
  Scene* scene = nullptr;
  View* view = nullptr;
  Camera* camera = nullptr;
  utils::Entity eye;
  utils::Entity quad;
  VertexBuffer* vertices = nullptr;
  IndexBuffer* indices = nullptr;
  std::vector<Material*> materials;
};

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
  engine->destroy(filament->quad);
  utils::EntityManager::get().destroy(filament->quad);
  for (Material* material : filament->materials) {
    engine->destroy(material);
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

extern "C" int32_t opsis_filament_pass(OpsisFilament* filament,
                                       int32_t material, const uint8_t* source,
                                       uint32_t width, uint32_t height,
                                       const float* floats, size_t float_count,
                                       uint8_t* out) {
  if (material < 0 ||
      static_cast<size_t>(material) >= filament->materials.size()) {
    return kFailed;
  }
  Engine* engine = filament->engine;
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
  MaterialInstance* instance =
      filament->materials[static_cast<size_t>(material)]->createInstance();
  instance->setParameter("source", texture,
                         TextureSampler(TextureSampler::MinFilter::LINEAR,
                                        TextureSampler::MagFilter::LINEAR));
  for (size_t i = 0; i < float_count; ++i) {
    instance->setParameter(("value" + std::to_string(i)).c_str(), floats[i]);
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
  engine->destroy(texture);
  return 0;
}
