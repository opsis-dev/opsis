/// <reference types="@electron-forge/plugin-vite/forge-vite-env" />
declare module "*.css";

type LocalFontData = {
  family: string;
  style: string;
  postscriptName: string;
  blob: () => Promise<Blob>;
};

interface Window {
  queryLocalFonts?: () => Promise<LocalFontData[]>;
}
