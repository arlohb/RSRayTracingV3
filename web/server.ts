import { serve } from "https://deno.land/std@0.116.0/http/server.ts";
import staticFiles from "https://deno.land/x/static_files@1.1.6/mod.ts";
import { mime } from "https://deno.land/x/mimetypes@v1.0.0/mod.ts";

const serveFiles = (req: Request) => staticFiles('public', {
  setHeaders: (headers, path, _stats) => {
    const mimeType = mime.getType(path) as string;
    console.log(mimeType);
    headers.append("Content-Type", mimeType);
  },
})({ 
    request: req, 
    respondWith: (r: Response) => r,
})

serve((req: Request) => serveFiles(req), { addr: ':3000' });
