// `Response.prototype.formData()` as Node runs it, for mm_core::form_data's differential tests.
export async function formData(contentType, body) {
  const form = await new Response(Buffer.from(body), { headers: { 'content-type': contentType } }).formData();
  const entries = [];
  for (const [name, value] of form) {
    entries.push(typeof value === 'string' ? { name, text: value }
      : { name, file: { name: value.name, type: value.type, bytes: new Uint8Array(await value.arrayBuffer()) } });
  }
  return entries;
}
