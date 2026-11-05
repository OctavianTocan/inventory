package com.octaviantocan.mymind.ui.shell

import androidx.annotation.DrawableRes
import com.octaviantocan.mymind.R

/**
 * The tabs of the bottom bar, in the order the original draws them.
 *
 * @property label Text under the icon.
 * @property lightIcon Icon used in the light appearance.
 * @property darkIcon Icon used in the dark appearance.
 */
enum class Tab(val label: String, @DrawableRes val lightIcon: Int, @DrawableRes val darkIcon: Int) {
    Everything("All", R.drawable.ic_tab_everything, R.drawable.ic_tab_everything),
    Spaces("Spaces", R.drawable.ic_tab_spaces, R.drawable.ic_tab_spaces),
}
