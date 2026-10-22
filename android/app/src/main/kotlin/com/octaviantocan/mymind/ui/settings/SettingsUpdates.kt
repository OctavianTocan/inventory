package com.octaviantocan.mymind.ui.settings

/**
 * What the profile sheet shows about app updates.
 *
 * @property visible False to leave the update rows out.
 * @property versionLabel The installed version, such as `Version 0.2.0 (2)`.
 * @property status Second line of the "Check for updates" row.
 * @property showAllow True to show the "Allow mymind to install updates" row.
 * @property onCheck Called when "Check for updates" is tapped.
 * @property onAllow Called when the allow row is tapped.
 * @property onChangelog Called when "What's new" is tapped.
 */
data class SettingsUpdates(
    val visible: Boolean = false,
    val versionLabel: String = "",
    val status: String = "",
    val showAllow: Boolean = false,
    val onCheck: () -> Unit = {},
    val onAllow: () -> Unit = {},
    val onChangelog: () -> Unit = {},
)
