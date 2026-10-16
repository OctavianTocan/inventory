package com.octaviantocan.mymind.data

/**
 * Resolves a media [path] from the API against [server].
 *
 * @param server server origin, with or without a trailing slash.
 * @param path server path such as `/media/...`, or an absolute URL that is returned unchanged.
 * @return an absolute URL.
 */
fun resolveMediaUrl(server: String, path: String): String = when {
    path.startsWith("http://") || path.startsWith("https://") -> path
    else -> server.trimEnd('/') + "/" + path.trimStart('/')
}
