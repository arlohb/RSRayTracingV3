import { createServer as createServerHTTP } from "http";
import { createServer as createServerHTTPS } from "https";
import { promises } from "fs";
import { lookup } from "mime-types";

const { readFile } = promises;

const port = 8080;
const webFolder = "dist";

const app = async (request, response) => {
  const url = request.url === "/" ? "/index.html" : request.url;
  const filename = `${process.cwd()}/${webFolder}${url}`;
  // lookup can deal with files in folders etc
  const mimeType = lookup(filename);
  console.log(url);

  let data;

  try {
    data = await readFile(filename);
  } catch {
    response.writeHead(404, {"Content-Type": "text/plain"});
    response.write("404 Not Found\n");
    response.end();
    return;
  }

  response.writeHead(200, {
    ...(mimeType !== false && {
      "Content-Type": mimeType
    }),
    "Cross-Origin-Embedder-Policy": "require-corp",
    "Cross-Origin-Opener-Policy": "same-origin",
  });
  response.write(data);
  response.end();
};

createServerHTTP(app).listen(port);

(async () => {
  let secrets;
  
  try {
    secrets = JSON.parse(await readFile("web/secrets.json"));
  } catch {
    console.log("secrets.json does not exist");
    return;
  }

  let key, cert;

  try {
    key = await readFile(secrets.key);
    cert = await readFile(secrets.cert);
  } catch {
    console.log("Key or cert does not exist");
    return;
  }
  
  try {
    createServerHTTPS({
      key,
      cert,
    }, app).listen(port + 1);
  } catch (e) {
    console.log(`HTTPS failed with error ${e}`);
  }
})();

console.log("Static file server running at http://localhost:" + port + "/\nCTRL + C to shutdown");