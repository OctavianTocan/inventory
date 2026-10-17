package com.octaviantocan.mymind.ui.shell

import androidx.annotation.RawRes
import com.octaviantocan.mymind.R

/**
 * The tabs of the bottom bar, in the order the original draws them.
 *
 * @property label Text under the icon.
 * @property lightIcon Lottie icon used in the light appearance.
 * @property darkIcon Lottie icon used in the dark appearance.
 */
enum class Tab(val label: String, @RawRes val lightIcon: Int, @RawRes val darkIcon: Int) {
    Everything("Everything", R.raw.everything_light, R.raw.everything_dark),
    Spaces("Spaces", R.raw.spaces_light, R.raw.spaces_dark),
    Serendipity("Serendipity", R.raw.serendipity_light, R.raw.serendipity_dark),
}
