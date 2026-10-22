package com.octaviantocan.mymind.ui.upload

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.menu.CardMenuGroup
import com.octaviantocan.mymind.ui.menu.CardMenuHeading
import com.octaviantocan.mymind.ui.menu.PullySheet
import com.octaviantocan.mymind.ui.menu.SheetRow
import com.octaviantocan.mymind.ui.menu.SheetTile

/** Label of the tile that opens the camera. */
const val TAKE_PHOTO = "Camera"

/** Label of the tile that opens the system photo picker. */
const val PHOTO_LIBRARY = "Photos"

/** Label of the tile that opens the system file picker. */
const val PICK_FILES = "Files"

/** Label of the row that starts a new space. */
const val NEW_SPACE = "New space"

/**
 * The sheet the + button opens, laid out like the Claude app's Add sheet: Camera, Photos and Files as tiles, then the
 * page's own additions below them.
 *
 * @param title The sheet's title.
 * @param onPhotos Called when Photos is tapped.
 * @param onFiles Called when Files is tapped.
 * @param onCamera Called when Camera is tapped; null leaves the tile out on phones without a camera app.
 * @param onClose Called once the sheet has closed by drag, Back or a tap outside.
 * @param onNewSpace Called when New space is tapped; null leaves the row out.
 */
@Composable
fun AddSheet(
    title: String,
    onPhotos: () -> Unit,
    onFiles: () -> Unit,
    onCamera: (() -> Unit)?,
    onClose: () -> Unit,
    onNewSpace: (() -> Unit)? = null,
) {
    PullySheet(title, onClose) { AddSheetContent(onPhotos, onFiles, onCamera, onNewSpace) }
}

/** The body of [AddSheet], also drawn by screenshot fixtures inside a bare panel. */
@Composable
internal fun ColumnScope.AddSheetContent(onPhotos: () -> Unit, onFiles: () -> Unit, onCamera: (() -> Unit)?, onNewSpace: (() -> Unit)?) {
    Row(Modifier.fillMaxWidth().padding(top = 20.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        if (onCamera != null) SheetTile(painterResource(R.drawable.baseline_camera_alt_24), TAKE_PHOTO, onCamera, Modifier.weight(1f))
        SheetTile(painterResource(R.drawable.baseline_photo_library_24), PHOTO_LIBRARY, onPhotos, Modifier.weight(1f))
        SheetTile(painterResource(R.drawable.ic_add_file), PICK_FILES, onFiles, Modifier.weight(1f))
    }
    if (onNewSpace != null) {
        CardMenuHeading("Organize")
        CardMenuGroup(Modifier.padding(top = 12.dp)) {
            SheetRow(painterResource(R.drawable.addtospaceicon), NEW_SPACE, onNewSpace, detail = "Keep related cards together")
        }
    }
}

/** MIME types the Files tile offers: what the library stores as cards. */
val PICKABLE_FILES = arrayOf("image/*", "video/*", "application/pdf")
