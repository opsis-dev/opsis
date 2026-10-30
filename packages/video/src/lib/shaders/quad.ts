const QUAD = new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]);

const VERTEX_SHADER = `#version 300 es
in vec2 a_pos;
out vec2 v_uv;
void main() {
  v_uv = vec2(a_pos.x * 0.5 + 0.5, 0.5 - a_pos.y * 0.5);
  gl_Position = vec4(a_pos, 0.0, 1.0);
}`;

function compileShader(gl: WebGL2RenderingContext, source: string, type: GLenum): WebGLShader {
  const shader = gl.createShader(type);
  if (!shader) {
    throw new Error("shader: failed to create shader");
  }
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const log = gl.getShaderInfoLog(shader);
    gl.deleteShader(shader);
    throw new Error(`shader: failed to compile shader: ${log}`);
  }
  return shader;
}

function createProgram(gl: WebGL2RenderingContext, fragment: string): WebGLProgram {
  const program = gl.createProgram();
  if (!program) {
    throw new Error("shader: failed to create program");
  }
  const vs = compileShader(gl, VERTEX_SHADER, gl.VERTEX_SHADER);
  const fs = compileShader(gl, fragment, gl.FRAGMENT_SHADER);
  gl.attachShader(program, vs);
  gl.attachShader(program, fs);
  gl.linkProgram(program);
  gl.deleteShader(vs);
  gl.deleteShader(fs);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    const log = gl.getProgramInfoLog(program);
    gl.deleteProgram(program);
    throw new Error(`shader: failed to link program: ${log}`);
  }
  return program;
}

function createSceneTexture(gl: WebGL2RenderingContext): WebGLTexture {
  const texture = gl.createTexture();
  if (!texture) {
    throw new Error("shader: failed to create texture");
  }
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texImage2D(
    gl.TEXTURE_2D,
    0,
    gl.RGBA,
    1,
    1,
    0,
    gl.RGBA,
    gl.UNSIGNED_BYTE,
    new Uint8Array([0, 0, 0, 0]),
  );
  return texture;
}

function webgl2(canvas: OffscreenCanvas | HTMLCanvasElement): WebGL2RenderingContext | null {
  return canvas instanceof HTMLCanvasElement
    ? canvas.getContext("webgl2", { premultipliedAlpha: true, preserveDrawingBuffer: true })
    : canvas.getContext("webgl2", { premultipliedAlpha: true });
}

export function createQuadSurface(canvas: OffscreenCanvas | HTMLCanvasElement, fragment: string) {
  const gl = webgl2(canvas);
  if (!gl) {
    throw new Error("shader: WebGL2 is unavailable");
  }
  return quadSurface(gl, fragment);
}

export function quadSurface(gl: WebGL2RenderingContext, fragment: string) {
  const program = createProgram(gl, fragment);
  const vao = gl.createVertexArray();
  gl.bindVertexArray(vao);
  const buffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.bufferData(gl.ARRAY_BUFFER, QUAD, gl.STATIC_DRAW);
  const aPos = gl.getAttribLocation(program, "a_pos");
  gl.enableVertexAttribArray(aPos);
  gl.vertexAttribPointer(aPos, 2, gl.FLOAT, false, 0, 0);

  const locations = new Map<string, WebGLUniformLocation | null>();
  function uniform(name: string) {
    const cached = locations.get(name);
    if (cached !== undefined) {
      return cached;
    }
    const location = gl.getUniformLocation(program, name);
    locations.set(name, location);
    return location;
  }

  function clear() {
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
  }

  const textures: WebGLTexture[] = [];
  function createTexture() {
    const texture = createSceneTexture(gl);
    textures.push(texture);
    return texture;
  }

  function begin(width: number, height: number) {
    gl.viewport(0, 0, width, height);
    clear();
    gl.useProgram(program);
    gl.bindVertexArray(vao);
  }

  function bindScene(
    image: TexImageSource | null,
    texture: WebGLTexture,
    unit: number,
    name: string,
  ) {
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, texture);
    if (image) {
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
    }
    gl.uniform1i(uniform(name), unit);
  }

  function draw() {
    return gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
  }

  function cleanup() {
    gl.deleteProgram(program);
    for (const texture of textures) {
      gl.deleteTexture(texture);
    }
    gl.deleteBuffer(buffer);
    gl.deleteVertexArray(vao);
  }

  return {
    gl,
    uniform,
    clear,
    createTexture,
    begin,
    bindScene,
    draw,
    cleanup,
  };
}
