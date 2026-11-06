package com.octaviantocan.mymind

import android.media.AudioManager
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.lifecycle.lifecycleScope
import com.octaviantocan.mymind.ui.MymindRoot
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // The volume keys set the media volume the app's sounds play at (share-moment-sounds).
        volumeControlStream = AudioManager.STREAM_MUSIC
        enableEdgeToEdge()
        val container = (application as MymindApp).container
        lifecycleScope.launch { container.session.restore() }
        setContent { MymindRoot(container) }
    }
}
