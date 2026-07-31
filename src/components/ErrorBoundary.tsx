import React, { Component, type ErrorInfo, type ReactNode } from "react";
import { reportFatal } from "@/lib/crashScreen";

interface ErrorBoundaryProps {
  children: ReactNode;
  /** Identifies the failure in the log and on the crash screen. */
  context: string;
  /**
   * What to show in place of the failed subtree. Omit for the boundary that
   * wraps the whole app: React unmounts the root on an uncaught render error,
   * so there is no UI left to preserve and the crash screen takes the window.
   */
  fallback?: ReactNode;
}

interface ErrorBoundaryState {
  failed: boolean;
}

/**
 * Stops one bad subtree from blanking the entire window.
 *
 * Without this, an uncaught render error unmounts the React root and leaves the
 * user with an empty window and no way to report what happened — see #1617.
 */
export class ErrorBoundary extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  state: ErrorBoundaryState = { failed: false };

  static getDerivedStateFromError(): ErrorBoundaryState {
    return { failed: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    const isRootBoundary = this.props.fallback === undefined;
    reportFatal(
      `${error.stack || error.message}\n\nComponent stack:${info.componentStack}`,
      this.props.context,
      // A root failure means the window really is blank, so paint over it even
      // if something was mounted a moment ago.
      { force: isRootBoundary },
    );
  }

  render(): ReactNode {
    if (this.state.failed) {
      return this.props.fallback ?? null;
    }
    return this.props.children;
  }
}
