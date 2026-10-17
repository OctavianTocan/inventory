// sharp is native code; on Cloudflare, images are processed in the media container (worker/container/).
export default function sharp() {
  throw new Error('sharp runs in the media container');
}
