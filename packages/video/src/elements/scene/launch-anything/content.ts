import { fontOr } from "#video/common/fonts.ts";
import { launchMedia } from "#video/elements/scene/launch-anything/assets.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export const shade = (color: string, percent: number) =>
  `color-mix(in srgb, ${color}, black ${percent}%)`;

export const tint = (color: string, percent: number) =>
  `color-mix(in srgb, ${color}, white ${percent}%)`;

export function launchScene(s: ElementSettings["launch-anything"]) {
  return {
    brandUrl: s.brandUrl,
    accent: s.accentColor,
    content: {
      opening: s.opening,
      subject: s.subject,
      ready: s.copyReady,
      next: s.copyNext,
      action: s.copyAction,
      proof: s.copyProof,
      industry: s.copyIndustry,
      promise: s.copyPromise,
      launch: s.copyLaunch,
      shoppers: s.copyShoppers,
      builder: s.copyBuilder,
      builderPrompt: s.copyBuilderPrompt,
      trading: s.copyTrading,
      loyalty: s.copyLoyalty,
      loyaltyPayoff: s.copyLoyaltyPayoff,
      integration: s.copyIntegration,
      integrations: s.copyIntegrations.filter((name) => name.trim()),
      shopName: s.copyShopName,
      shopHeadline: s.copyShopHeadline,
      shopDetail: s.copyShopDetail,
      shopAction: s.copyShopAction,
      connections: s.copyConnections,
      appName: s.copyAppName,
      dashboardMenu: s.copyDashboardMenu,
      dashboard: s.copyDashboard,
      dashboardPeriod: s.copyDashboardPeriod,
      dashboardNote: s.copyDashboardNote,
      dashboardStats: s.dashboardStats,
      momentum: s.copyMomentum,
      activityTitle: s.copyActivityTitle,
      dashboardActivity: s.dashboardActivity,
      shopNote: s.copyShopNote,
      reviewTitle: s.copyReviewTitle,
      reviewProject: s.copyReviewProject,
      reviewButtons: s.copyReviewButtons,
      reviewRows: s.reviewRows,
      ideasCard: s.copyIdeasCard,
      ideasNote: s.copyIdeasNote,
    },
    media: {
      desk: s.photoDesk || launchMedia.desk,
      horizon: s.photoHorizon || launchMedia.horizon,
      rocket: s.photoRocket || launchMedia.rocket,
      earth: s.photoEarth || launchMedia.earth,
    },
    screenImages: {
      dashboard: s.screenDashboard,
      shoppers: s.screenShoppers,
      exchange: s.screenExchange,
      builder: s.screenBuilder,
      trading: s.screenTrading,
      loyalty: s.screenLoyalty,
      integrations: s.screenIntegrations,
    },
    logos: s.logoImages.map((logo) => logo.image).filter((src) => src.trim()),
    logoSrc: s.logoSrc || undefined,
    palette: {
      night: s.nightColor,
      paper: s.paperColor,
      lightText: s.lightTextColor,
      ink: s.inkColor,
      sage: s.sageColor,
    },
    look: {
      font: fontOr(s.fontFamily, "main"),
      openingShimmer: s.openingShimmer,
      openingSize: s.openingSize,
      headlineWeight: s.headlineWeight,
      textSize: s.textSize,
      photoDimming: s.photoDimming,
      logoSize: s.logoSize,
    },
  };
}

export type LaunchScene = ReturnType<typeof launchScene>;
export type SceneProps = { scene: LaunchScene; t: number };
