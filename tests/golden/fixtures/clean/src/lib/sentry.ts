// P001 fixture: minimal Sentry config — satisfies INV-005
// security-gate.sh:116 checks for beforeSend/beforeBreadcrumb
export function initSentry() {
  return {
    beforeSend: (event: unknown) => event,
    beforeBreadcrumb: (breadcrumb: unknown) => breadcrumb,
  };
}
