import { Dialog, DialogContent, DialogTitle, DialogTrigger } from "@/components/ui/dialog";

export function ImagePreview({ src, alt }: { src: string; alt: string }) {
  return (
    <Dialog>
      <DialogTrigger
        render={
          <button
            type="button"
            aria-label={`Enlarge ${alt}`}
            className="block w-full cursor-zoom-in overflow-hidden rounded-md"
          />
        }
      >
        <img src={src} alt={alt} className="w-full" />
      </DialogTrigger>
      <DialogContent className="w-auto max-w-[94vw] p-2 sm:max-w-[94vw]">
        <DialogTitle className="sr-only">{alt}</DialogTitle>
        <img src={src} alt={alt} className="max-h-[88vh] max-w-full rounded-md object-contain" />
      </DialogContent>
    </Dialog>
  );
}
