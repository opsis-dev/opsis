vec2 paperPosition() {
    vec2 uv = getUV0();
    return vec2(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
}

vec2 paperResolution() {
    return materialParams.size;
}

vec3 paperBoxSize(float boxRatio, vec2 givenBoxSize) {
    vec2 resolution = paperResolution();
    vec2 box = vec2(0.0);
    box.x = boxRatio * min(givenBoxSize.x / boxRatio, givenBoxSize.y);
    float noFitBoxWidth = box.x;
    if (materialParams.fit == 1.0) {
        box.x = boxRatio * min(resolution.x / boxRatio, resolution.y);
    } else if (materialParams.fit == 2.0) {
        box.x = boxRatio * max(resolution.x / boxRatio, resolution.y);
    }
    box.y = box.x / boxRatio;
    return vec3(box, noFitBoxWidth);
}

mat2 paperRotation() {
    float r = materialParams.rotation * 3.14159265358979323846 / 180.0;
    return mat2(cos(r), sin(r), -sin(r), cos(r));
}

vec2 paperGivenSize() {
    vec2 resolution = paperResolution();
    vec2 given = max(vec2(materialParams.worldWidth, materialParams.worldHeight), vec2(1.0)) * materialParams.scale;
    return vec2(
        (materialParams.worldWidth == 0.0) ? resolution.x : given.x,
        (materialParams.worldHeight == 0.0) ? resolution.y : given.y);
}

vec2 paperOrigin() {
    return vec2(0.5 - materialParams.originX, materialParams.originY - 0.5);
}

vec2 paperOffset() {
    return vec2(-materialParams.offsetX, materialParams.offsetY);
}

vec2 paperObjectUV() {
    vec2 uv = paperPosition() * 0.5;
    vec2 objectBoxSize = paperBoxSize(1.0, paperGivenSize()).xy;
    vec2 objectWorldScale = paperResolution() / objectBoxSize;
    vec2 objectUV = uv * objectWorldScale;
    objectUV += paperOrigin() * (objectWorldScale - 1.0);
    objectUV += paperOffset();
    objectUV /= materialParams.sizingScale;
    return paperRotation() * objectUV;
}

vec2 paperPatternUV() {
    vec2 resolution = paperResolution();
    vec2 uv = paperPosition() * 0.5;
    vec2 given = paperGivenSize();
    float patternBoxRatio = given.x / given.y;
    vec3 boxSizeData = paperBoxSize(patternBoxRatio, given);
    vec2 patternBoxSize = boxSizeData.xy;
    vec2 patternBoxScale = resolution / patternBoxSize;
    vec2 origin = paperOrigin();
    vec2 patternUV = uv;
    patternUV += paperOffset() / patternBoxScale;
    patternUV += origin;
    patternUV -= origin / patternBoxScale;
    patternUV *= resolution;
    patternUV /= materialParams.scale;
    if (materialParams.fit > 0.0) {
        patternUV *= (boxSizeData.z / patternBoxSize.x);
    }
    patternUV /= materialParams.sizingScale;
    patternUV = paperRotation() * patternUV;
    patternUV += origin / patternBoxScale;
    patternUV -= origin;
    return patternUV * 0.01;
}

uvec2 paperPcg(uvec2 v) {
    v = v * 1664525u + 1013904223u;
    v.x += v.y * 1664525u;
    v.y += v.x * 1664525u;
    v = v ^ (v >> 16u);
    v.x += v.y * 1664525u;
    v.y += v.x * 1664525u;
    v = v ^ (v >> 16u);
    return v;
}

vec3 paperBandingFix(vec3 color) {
    uvec2 pixel = uvec2(floor(getUV0() * paperResolution()));
    return color + 1.0 / 256.0 * (float(paperPcg(pixel).x) / 4294967295.0 - 0.5);
}

vec2 paperObjectBoxSize() {
    return paperBoxSize(1.0, paperGivenSize()).xy;
}

vec2 paperPatternBoxSize() {
    vec2 given = paperGivenSize();
    return paperBoxSize(given.x / given.y, given).xy;
}
