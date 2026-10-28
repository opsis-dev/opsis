import { centered, component, fitScreen, listsFromRows, marksFromImages, nested, painted, placedAt, playsAtEnd, pointsFromSpots, presentation, sequencer, themed, ticksFromWords, treeFrom } from "#video/registry/framing.tsx";
import type { ElementImpl } from "#video/registry/types.ts";
import { Placeholder } from "#video/core/placeholder.tsx";
import { StillElement } from "#video/core/still.tsx";
import { AuroraBg } from "#video/elements/background/aurora-bg.tsx";
import { Backdrop } from "#video/elements/background/backdrop.tsx";
import { Gradient } from "#video/elements/background/gradient.tsx";
import { NoiseField } from "#video/elements/background/noise.tsx";
import { Paper } from "#video/elements/background/paper.tsx";
import { ParticleField } from "#video/elements/background/particle-field.tsx";
import { ShaderDithering } from "#video/elements/background/shader-dithering.tsx";
import { ShaderGodRays } from "#video/elements/background/shader-god-rays.tsx";
import { ShaderGrainGradient } from "#video/elements/background/shader-grain-gradient.tsx";
import { ShaderMeshGradient } from "#video/elements/background/shader-mesh-gradient.tsx";
import { ShaderPerlinNoise } from "#video/elements/background/shader-perlin-noise.tsx";
import { ShaderSwirl } from "#video/elements/background/shader-swirl.tsx";
import { ShaderWarp } from "#video/elements/background/shader-warp.tsx";
import { Shapes3D } from "#video/elements/background/shapes-3d.tsx";
import { Solid } from "#video/elements/background/solid.tsx";
import { Starburst } from "#video/elements/background/starburst.tsx";
import { VideoLoop } from "#video/elements/background/video-loop.tsx";
import { AnimatedBarChart } from "#video/elements/data/animated-bar-chart.tsx";
import { AnimatedLineChart } from "#video/elements/data/animated-line-chart.tsx";
import { BarChartRace } from "#video/elements/data/bar-chart-race.tsx";
import { ComparisonBars } from "#video/elements/data/comparison-bars.tsx";
import { DonutChart } from "#video/elements/data/donut-chart.tsx";
import { FunnelChart } from "#video/elements/data/funnel-chart.tsx";
import { GaugeDial } from "#video/elements/data/gauge-dial.tsx";
import { MetricTicker } from "#video/elements/data/metric-ticker/index.tsx";
import { RollingNumber } from "#video/elements/data/rolling-number.tsx";
import { StatCard } from "#video/elements/data/stat-card/index.tsx";
import { DeviceMockupZoom } from "#video/elements/device/device-mockup-zoom/index.tsx";
import { LaptopFrame } from "#video/elements/device/laptop-frame.tsx";
import { MultiDeviceLineup } from "#video/elements/device/multi-device-lineup.tsx";
import { PhoneFrame } from "#video/elements/device/phone-frame.tsx";
import { Screen3D } from "#video/elements/device/screen-3d.tsx";
import { ScreenRecording } from "#video/elements/device/screen-recording.tsx";
import { CameraLens } from "#video/elements/effect/camera-lens.tsx";
import { CrtScreen } from "#video/elements/effect/crt-screen.tsx";
import { HalftonePrint } from "#video/elements/effect/halftone-print.tsx";
import { Mosaic } from "#video/elements/effect/mosaic/index.tsx";
import { Polaroid } from "#video/elements/effect/polaroid.tsx";
import { SpeedLines } from "#video/elements/effect/speed-lines.tsx";
import { TvPowerOff } from "#video/elements/effect/tv-power-off.tsx";
import { VhsFilter } from "#video/elements/effect/vhs-filter.tsx";
import { ImageElement } from "#video/elements/media/image.tsx";
import { VideoElement } from "#video/elements/media/video.tsx";
import { Callout } from "#video/elements/overlay/callout.tsx";
import { Captions } from "#video/elements/overlay/captions.tsx";
import { Confetti } from "#video/elements/overlay/confetti.tsx";
import { InkArrow } from "#video/elements/overlay/ink-arrow.tsx";
import { InkUnderline } from "#video/elements/overlay/ink-underline.tsx";
import { InlineHighlight } from "#video/elements/overlay/inline-highlight.tsx";
import { Keystroke } from "#video/elements/overlay/keystroke.tsx";
import { LightLeakElement } from "#video/elements/overlay/light-leak.tsx";
import { LottieElement } from "#video/elements/overlay/lottie.tsx";
import { LowerThird } from "#video/elements/overlay/lower-third.tsx";
import { MarkerHighlight } from "#video/elements/overlay/marker-highlight.tsx";
import { NotificationStack } from "#video/elements/overlay/notification-stack/index.tsx";
import { RadialBurst } from "#video/elements/overlay/radial-burst.tsx";
import { ScribbleCircle } from "#video/elements/overlay/scribble-circle.tsx";
import { AiPromptFlow } from "#video/elements/scene/ai-prompt-flow/index.tsx";
import { Card3D } from "#video/elements/scene/card-3d.tsx";
import { ChangelogEntry } from "#video/elements/scene/changelog-entry/index.tsx";
import { CommandMenu } from "#video/elements/scene/command-menu/index.tsx";
import { ComparisonTable } from "#video/elements/scene/comparison-table/index.tsx";
import { DataStory } from "#video/elements/scene/data-story/index.tsx";
import { EndCard } from "#video/elements/scene/end-card/index.tsx";
import { HeroLaunch } from "#video/elements/scene/hero-launch.tsx";
import { ProductShowcase } from "#video/elements/scene/launch-anything/index.tsx";
import { Logo3D } from "#video/elements/scene/logo-3d.tsx";
import { LogoAssemble } from "#video/elements/scene/logo-assemble.tsx";
import { LogoDrift } from "#video/elements/scene/logo-drift.tsx";
import { LogoReveal } from "#video/elements/scene/logo-reveal/index.tsx";
import { LogoWall } from "#video/elements/scene/logo-wall/index.tsx";
import { Model3D } from "#video/elements/scene/model-3d.tsx";
import { PricingCard } from "#video/elements/scene/pricing-card/index.tsx";
import { ReleaseTeaser } from "#video/elements/scene/release-teaser/index.tsx";
import { SignupFlow } from "#video/elements/scene/signup-flow/index.tsx";
import { SpringSettleItem, SpringSettleScenes } from "#video/elements/scene/spring-settle/index.tsx";
import { AnnounceTitle } from "#video/elements/text/announce-title.tsx";
import { GradientScaleCutText } from "#video/elements/text/gradient-scale-cut-text.tsx";
import { HandwritingText } from "#video/elements/text/handwriting-text.tsx";
import { KineticCenterBuild } from "#video/elements/text/kinetic-center-build.tsx";
import { KineticHeadline } from "#video/elements/text/kinetic-headline.tsx";
import { LineByLineSlide } from "#video/elements/text/line-by-line-slide.tsx";
import { MaskRevealUp } from "#video/elements/text/mask-reveal-up.tsx";
import { MatrixDecode } from "#video/elements/text/matrix-decode.tsx";
import { NeonFlickerText } from "#video/elements/text/neon-flicker-text.tsx";
import { PunchLines } from "#video/elements/text/punch-lines.tsx";
import { RGBGlitchText } from "#video/elements/text/rgb-glitch-text.tsx";
import { ShimmerSweep } from "#video/elements/text/shimmer-sweep.tsx";
import { SlotMachineRoll } from "#video/elements/text/slot-machine-roll.tsx";
import { SoftBlurIn } from "#video/elements/text/soft-blur-in.tsx";
import { StaggeredFadeUp } from "#video/elements/text/staggered-fade-up.tsx";
import { StatusCycle } from "#video/elements/text/status-cycle.tsx";
import { StrikethroughReplace } from "#video/elements/text/strikethrough-replace.tsx";
import { Text3D } from "#video/elements/text/text-3d.tsx";
import { TextHighlight } from "#video/elements/text/text-highlight.tsx";
import { TextMaskVideo } from "#video/elements/text/text-mask-video.tsx";
import { Title } from "#video/elements/text/title.tsx";
import { TrackingIn } from "#video/elements/text/tracking-in.tsx";
import { Typewriter } from "#video/elements/text/typewriter.tsx";
import { WordPush } from "#video/elements/text/word-push.tsx";
import { chromaticAberrationWipe } from "#video/elements/transition/chromatic-aberration-wipe.tsx";
import { clockWipe } from "#video/elements/transition/clock-wipe.ts";
import { ditherDissolve } from "#video/elements/transition/dither-dissolve.tsx";
import { fade } from "#video/elements/transition/fade.ts";
import { flip } from "#video/elements/transition/flip.ts";
import { focusPull } from "#video/elements/transition/focus-pull.tsx";
import { frostedGlassWipe } from "#video/elements/transition/frosted-glass-wipe.tsx";
import { glitchCut } from "#video/elements/transition/glitch-cut.tsx";
import { grainDissolve } from "#video/elements/transition/grain-dissolve.tsx";
import { gridPixelateWipe } from "#video/elements/transition/grid-pixelate-wipe.tsx";
import { iris } from "#video/elements/transition/iris.ts";
import { lensZoom } from "#video/elements/transition/lens-zoom.tsx";
import { pageTurn } from "#video/elements/transition/page-turn.tsx";
import { particleDissolve } from "#video/elements/transition/particle-dissolve.tsx";
import { pushThrough } from "#video/elements/transition/push-through.tsx";
import { rippleZoom } from "#video/elements/transition/ripple-zoom.tsx";
import { slide } from "#video/elements/transition/slide.ts";
import { smokeDissolve } from "#video/elements/transition/smoke-dissolve/index.tsx";
import { swirlDissolve } from "#video/elements/transition/swirl-dissolve.tsx";
import { blinds } from "#video/elements/transition/transition-blinds.tsx";
import { morphShape } from "#video/elements/transition/transition-morph-shape/index.tsx";
import { waveWipe } from "#video/elements/transition/wave-wipe.tsx";
import { whipPan } from "#video/elements/transition/whip-pan.tsx";
import { wipe } from "#video/elements/transition/wipe.ts";
import { zoomBlur } from "#video/elements/transition/zoom-blur.tsx";
import { AgentRun } from "#video/elements/ui/agent-run.tsx";
import { CheckList } from "#video/elements/ui/check-list.tsx";
import { CodeDiffWipe } from "#video/elements/ui/code-diff-wipe/index.tsx";
import { CodeMorph } from "#video/elements/ui/code-morph.tsx";
import { CodeReveal } from "#video/elements/ui/code-reveal/index.tsx";
import { CommitGraph } from "#video/elements/ui/commit-graph/index.tsx";
import { DragDropFlow } from "#video/elements/ui/drag-drop-flow/index.tsx";
import { FileTreeReveal } from "#video/elements/ui/file-tree-reveal/index.tsx";
import { GlassCodeBlock } from "#video/elements/ui/glass-code-block.tsx";
import { KanbanMove } from "#video/elements/ui/kanban-move/index.tsx";
import { LogoEnter } from "#video/elements/ui/logo-enter.tsx";
import { SearchReveal } from "#video/elements/ui/search-reveal.tsx";
import { TerminalSimulator } from "#video/elements/ui/terminal-simulator.tsx";

export const ELEMENTS: Readonly<Record<string, ElementImpl>> = {
  "soft-blur-in": painted(component(SoftBlurIn)),
  typewriter: painted(component(Typewriter)),
  "shimmer-sweep": painted(component(ShimmerSweep)),
  "matrix-decode": painted(component(MatrixDecode)),
  "slot-machine-roll": painted(component(SlotMachineRoll)),
  "staggered-fade-up": painted(component(StaggeredFadeUp)),
  "kinetic-center-build": painted(component(KineticCenterBuild)),
  "mask-reveal-up": painted(component(MaskRevealUp)),
  "tracking-in": painted(component(TrackingIn)),
  "word-push": painted(component(WordPush)),
  "rgb-glitch-text": painted(component(RGBGlitchText)),
  "gradient-scale-cut-text": component(GradientScaleCutText),
  "strikethrough-replace": painted(component(StrikethroughReplace)),
  "line-by-line-slide": painted(component(LineByLineSlide)),
  "marker-highlight": painted(component(MarkerHighlight)),
  "inline-highlight": painted(component(InlineHighlight)),
  "ink-underline": placedAt(component(InkUnderline)),
  "scribble-circle": placedAt(component(ScribbleCircle)),
  "ink-arrow": pointsFromSpots(component(InkArrow)),
  keystroke: component(Keystroke),
  confetti: component(Confetti),
  "radial-burst": component(RadialBurst),
  "whip-pan": presentation(whipPan),
  "push-through": presentation(pushThrough),
  "zoom-blur": presentation(zoomBlur),
  "focus-pull": presentation(focusPull),
  "lens-zoom": presentation(lensZoom),
  "ripple-zoom": presentation(rippleZoom),
  "wave-wipe": presentation(waveWipe),
  "smoke-dissolve": presentation(smokeDissolve),
  "glitch-cut": presentation(glitchCut),
  "page-turn": presentation(pageTurn),
  "grain-dissolve": presentation(grainDissolve),
  "dither-dissolve": presentation(ditherDissolve),
  "swirl-dissolve": presentation(swirlDissolve),
  "particle-dissolve": presentation(particleDissolve),
  "shader-mesh-gradient": component(ShaderMeshGradient),
  "shader-grain-gradient": component(ShaderGrainGradient),
  "shader-warp": component(ShaderWarp),
  "shader-swirl": component(ShaderSwirl),
  "shader-god-rays": component(ShaderGodRays),
  "shader-perlin-noise": component(ShaderPerlinNoise),
  "shader-dithering": component(ShaderDithering),
  "crt-screen": component(CrtScreen),
  "vhs-filter": component(VhsFilter),
  "halftone-print": component(HalftonePrint),
  "camera-lens": component(CameraLens),
  polaroid: centered(component(Polaroid)),
  "tv-power-off": playsAtEnd(component(TvPowerOff)),
  "speed-lines": component(SpeedLines),
  "spring-settle": sequencer(SpringSettleScenes, SpringSettleItem),
  "terminal-simulator": component(TerminalSimulator),
  "glass-code-block": component(GlassCodeBlock),
  "code-morph": component(CodeMorph),
  "logo-enter": marksFromImages(component(LogoEnter), {"from":"logos","to":"logos"}),
  "check-list": painted(centered(component(CheckList))),
  "search-reveal": component(SearchReveal),
  "agent-run": nested(component(AgentRun), {"runTitle":"run.title","runPrompt":"run.prompt","runPlan":"run.plan","runSteps":"run.steps","runAnswer":"run.answer"}),
  "animated-bar-chart": listsFromRows(component(AnimatedBarChart), {"bars":{"label":"labels","value":"data"}}),
  "animated-line-chart": component(AnimatedLineChart),
  "rolling-number": painted(component(RollingNumber)),
  "signup-flow": themed(component(SignupFlow), "context"),
  "ai-prompt-flow": themed(component(AiPromptFlow), "context"),
  "command-menu": themed(component(CommandMenu), "context"),
  "release-teaser": nested(component(ReleaseTeaser), {"themeBackground":"theme.background","themeSurface":"theme.surface","themeForeground":"theme.foreground","themeMuted":"theme.muted"}),
  "launch-anything": listsFromRows(nested(component(ProductShowcase), {"copyReady":"content.ready","copyNext":"content.next","copyAction":"content.action","copyProof":"content.proof","copyIndustry":"content.industry","copyPromise":"content.promise","copyLaunch":"content.launch","copyShoppers":"content.shoppers","copyShopName":"content.shopName","copyShopHeadline":"content.shopHeadline","copyShopDetail":"content.shopDetail","copyShopAction":"content.shopAction","copyBuilder":"content.builder","copyBuilderPrompt":"content.builderPrompt","copyTrading":"content.trading","copyLoyalty":"content.loyalty","copyLoyaltyPayoff":"content.loyaltyPayoff","copyIntegration":"content.integration","copyConnections":"content.connections","copyIntegrations":"content.integrations","screenDashboard":"screenImages.dashboard","screenShoppers":"screenImages.shoppers","screenExchange":"screenImages.exchange","screenBuilder":"screenImages.builder","screenTrading":"screenImages.trading","screenLoyalty":"screenImages.loyalty","screenIntegrations":"screenImages.integrations","photoDesk":"media.desk","photoHorizon":"media.horizon","photoRocket":"media.rocket","photoEarth":"media.earth"}), {"logoImages":{"image":"logos"}}),
  "laptop-frame": themed(component(LaptopFrame), "prop"),
  "phone-frame": themed(component(PhoneFrame), "prop"),
  "screen-recording": themed(component(ScreenRecording), "prop"),
  "announce-title": component(AnnounceTitle),
  "punch-lines": themed(component(PunchLines), "prop"),
  "text-highlight": themed(component(TextHighlight), "prop"),
  "status-cycle": themed(component(StatusCycle), "prop"),
  "logo-assemble": themed(listsFromRows(component(LogoAssemble), {"images":{"image":"images"}}), "prop"),
  "logo-drift": themed(component(LogoDrift), "prop"),
  "hero-launch": themed(component(HeroLaunch), "prop"),
  "grid-pixelate-wipe": presentation(gridPixelateWipe),
  "frosted-glass-wipe": presentation(frostedGlassWipe),
  "chromatic-aberration-wipe": presentation(chromaticAberrationWipe),
  "transition-blinds": presentation(blinds),
  "transition-morph-shape": presentation(morphShape),
  "stat-card": component(StatCard),
  "metric-ticker": component(MetricTicker),
  "funnel-chart": centered(component(FunnelChart)),
  "donut-chart": centered(component(DonutChart)),
  "comparison-bars": centered(component(ComparisonBars)),
  "bar-chart-race": centered(component(BarChartRace)),
  "gauge-dial": centered(component(GaugeDial)),
  "data-story": component(DataStory),
  "logo-reveal": component(LogoReveal),
  "pricing-card": component(PricingCard),
  "comparison-table": ticksFromWords(component(ComparisonTable), "rows"),
  "changelog-entry": nested(component(ChangelogEntry), {"kindAdded":"kindColors.added","kindFixed":"kindColors.fixed","kindChanged":"kindColors.changed","kindRemoved":"kindColors.removed"}),
  "logo-wall": component(LogoWall),
  "end-card": component(EndCard),
  "code-reveal": component(CodeReveal),
  "code-diff-wipe": component(CodeDiffWipe),
  "kanban-move": component(KanbanMove),
  "file-tree-reveal": treeFrom(component(FileTreeReveal), "nodes"),
  "drag-drop-flow": component(DragDropFlow),
  "commit-graph": component(CommitGraph),
  "notification-stack": component(NotificationStack),
  "device-mockup-zoom": fitScreen(component(DeviceMockupZoom)),
  "multi-device-lineup": centered(fitScreen(nested(component(MultiDeviceLineup), {"phoneLabel":"phone.label","tabletLabel":"tablet.label","laptopLabel":"laptop.label"}))),
  "handwriting-text": centered(component(HandwritingText)),
  "neon-flicker-text": centered(component(NeonFlickerText)),
  "text-mask-video": centered(component(TextMaskVideo)),
  "aurora-bg": component(AuroraBg),
  "particle-field": component(ParticleField),
  placeholder: component(Placeholder),
  still: component(StillElement),
  title: component(Title),
  "kinetic-headline": component(KineticHeadline),
  "lower-third": component(LowerThird),
  callout: component(Callout),
  captions: component(Captions),
  image: component(ImageElement),
  lottie: component(LottieElement),
  mosaic: component(Mosaic),
  video: component(VideoElement),
  "video-loop": component(VideoLoop),
  solid: component(Solid),
  gradient: component(Gradient),
  noise: component(NoiseField),
  starburst: component(Starburst),
  paper: component(Paper),
  backdrop: component(Backdrop),
  "light-leak": component(LightLeakElement),
  "logo-3d": component(Logo3D),
  "card-3d": component(Card3D),
  "text-3d": component(Text3D),
  "model-3d": component(Model3D),
  "screen-3d": component(Screen3D),
  "shapes-3d": component(Shapes3D),
  fade: fade,
  slide: slide,
  wipe: wipe,
  flip: flip,
  "clock-wipe": clockWipe,
  iris: iris,
};
