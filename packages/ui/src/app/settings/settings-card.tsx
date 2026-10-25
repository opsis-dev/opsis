import type { ReactNode } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export function SettingsCard({
  title,
  badges,
  description,
  contentClassName,
  footer,
  children,
}: {
  title: ReactNode;
  badges?: ReactNode;
  description?: ReactNode;
  contentClassName?: string;
  footer?: ReactNode;
  children?: ReactNode;
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className={badges ? "flex items-center gap-2" : undefined}>
          {title}
          {badges}
        </CardTitle>
        {description ? <CardDescription>{description}</CardDescription> : null}
      </CardHeader>
      {children ? <CardContent className={contentClassName}>{children}</CardContent> : null}
      {footer ? <CardFooter className="flex-wrap gap-2">{footer}</CardFooter> : null}
    </Card>
  );
}
