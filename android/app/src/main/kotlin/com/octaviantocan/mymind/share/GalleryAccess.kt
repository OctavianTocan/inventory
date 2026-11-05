package com.octaviantocan.mymind.share

import android.Manifest
import android.content.Context
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import androidx.core.content.ContextCompat
import com.octaviantocan.mymind.data.GOOGLE_PHOTOS_AUTHORITY
import com.octaviantocan.mymind.data.mediaStoreUriOf

/** ThyMind's own access to the Gallery, which lets a shared photo or video upload without a copy on the phone. */
object GalleryAccess {
    private const val PREFERENCES = "gallery-access"
    private const val ASKED = "asked"
    private const val GOOGLE_PHOTOS_PACKAGE = "com.google.android.apps.photos"

    /** The permissions that give full photo and video access on this Android version. */
    val permissions: Array<String>
        get() = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            arrayOf(Manifest.permission.READ_MEDIA_IMAGES, Manifest.permission.READ_MEDIA_VIDEO)
        } else {
            arrayOf(Manifest.permission.READ_EXTERNAL_STORAGE)
        }

    /**
     * Whether ThyMind may read every photo and video; access to only some of them still copies.
     *
     * @param context Any context of the app.
     * @return True when every permission in [permissions] is granted.
     */
    fun granted(context: Context): Boolean =
        permissions.all { ContextCompat.checkSelfPermission(context, it) == PackageManager.PERMISSION_GRANTED }

    /**
     * Whether to ask for access before saving a share: once ever, and only when a Gallery item is shared.
     *
     * @param context Any context of the app.
     * @param items What was shared.
     * @return True when the question should be shown now.
     */
    fun shouldAsk(context: Context, items: List<ShareItem>): Boolean =
        !granted(context) &&
            !context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE).getBoolean(ASKED, false) &&
            items.any { it is ShareItem.Stream && mediaStoreUriOf(Uri.parse(it.uri)) != null }

    /**
     * Whether Google Photos' link authority belongs to the Photos app the phone came with, so a Gallery entry wrapped
     * in such a link was really shared by Photos.
     *
     * @param context Any context of the app.
     * @return True for the preinstalled Google Photos.
     */
    fun photosIsGenuine(context: Context): Boolean {
        val provider = context.packageManager.resolveContentProvider(GOOGLE_PHOTOS_AUTHORITY, 0) ?: return false
        val system = ApplicationInfo.FLAG_SYSTEM or ApplicationInfo.FLAG_UPDATED_SYSTEM_APP
        return provider.packageName == GOOGLE_PHOTOS_PACKAGE && provider.applicationInfo.flags and system != 0
    }

    /**
     * Whether a shared file may be read at all: a Gallery entry, which ThyMind could read on its own once it has
     * access, only when the share granted it, which Android does only when the sending app could read it itself.
     *
     * @param uri The shared file.
     * @param shareGranted Whether the share carried a read grant.
     * @return False for a Gallery entry sent without a grant.
     */
    fun mayRead(uri: Uri, shareGranted: Boolean): Boolean = shareGranted || mediaStoreUriOf(uri) == null

    /**
     * Remembers that the question was shown, so it is not shown again.
     *
     * @param context Any context of the app.
     */
    fun markAsked(context: Context) {
        context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE).edit().putBoolean(ASKED, true).apply()
    }
}
