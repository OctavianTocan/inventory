package com.octaviantocan.mymind

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
        enableEdgeToEdge()
        val container = (application as MymindApp).container
        lifecycleScope.launch { container.session.restore() }
        setContent { MymindRoot(container) }
    }

    override fun onStart() {
        super.onStart()
        (application as MymindApp).container.updater.foreground = true
    }

    override fun onStop() {
        (application as MymindApp).container.updater.foreground = false
        super.onStop()
    }
}
