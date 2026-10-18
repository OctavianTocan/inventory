package com.octaviantocan.mymind.ui.shell

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.size
import androidx.compose.material.BottomNavigation
import androidx.compose.material.BottomNavigationItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.ui.theme.MymindTheme

/**
 * Bottom tab bar drawn over the screen content, with a gradient that fades the content out behind it. Every tap gives
 * the confirm haptic, as the original app's tab bar does.
 *
 * @param selected Tab shown as selected.
 * @param onSelect Called with the tab the user tapped.
 * @param modifier Modifier for the bar's 128dp tall container.
 */
@Composable
fun MymindTabBar(selected: Tab, onSelect: (Tab) -> Unit, modifier: Modifier = Modifier) {
    val fade = MymindTheme.colors.tabBarFade
    val haptic = LocalHapticFeedback.current
    Box(
        modifier
            .fillMaxWidth()
            .height(128.dp)
            .background(Brush.verticalGradient(0f to Color.Transparent, 0.25f to fade)),
        contentAlignment = Alignment.BottomCenter,
    ) {
        BottomNavigation(
            windowInsets = WindowInsets.navigationBars,
            backgroundColor = Color.Transparent,
            contentColor = Color.Transparent,
            elevation = 0.dp,
        ) {
            Tab.entries.forEach { tab ->
                TabBarItem(tab, tab == selected) {
                    haptic.performHapticFeedback(HapticFeedbackType.Confirm)
                    onSelect(tab)
                }
            }
        }
    }
}

@Composable
private fun RowScope.TabBarItem(tab: Tab, selected: Boolean, onClick: () -> Unit) {
    val colors = MymindTheme.colors
    val labelColor by animateColorAsState(if (selected) colors.tabSelected else colors.tabUnselected)
    BottomNavigationItem(
        selected = selected,
        onClick = onClick,
        icon = { TabIcon(tab, selected) },
        label = {
            Text(
                tab.label,
                style = TextStyle(
                    color = labelColor,
                    fontSize = 13.sp,
                    fontFamily = MymindTheme.fonts.nunito,
                    lineHeight = 18.sp,
                ),
            )
        },
        selectedContentColor = colors.tabSelected,
        unselectedContentColor = colors.tabUnselected,
    )
}

@Composable
private fun TabIcon(tab: Tab, selected: Boolean) {
    val colors = MymindTheme.colors
    val icon = if (colors.dark) tab.darkIcon else tab.lightIcon
    val tint by animateColorAsState(if (selected) colors.tabSelected else colors.tabUnselected)
    Column {
        Spacer(Modifier.height(16.dp))
        Icon(painterResource(icon), contentDescription = null, modifier = Modifier.size(24.dp), tint = tint)
    }
}
