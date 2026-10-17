package com.octaviantocan.mymind.data

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.distinctUntilChanged

/**
 * Reports whether the phone has a network that offers internet access.
 *
 * @param context Any context; the application context is kept.
 */
class Connectivity(context: Context) {
    private val manager = context.applicationContext.getSystemService(ConnectivityManager::class.java)

    /** @return True when the active network offers internet access right now. */
    fun isOnline(): Boolean {
        val network = manager.activeNetwork ?: return false
        return manager.getNetworkCapabilities(network)?.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) == true
    }

    /** Emits the current state on collection, then again whenever it changes. */
    val online: Flow<Boolean> = callbackFlow {
        val networks = mutableSetOf<Network>()
        val callback = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) {
                synchronized(networks) {
                    networks.add(network)
                    trySend(true)
                }
            }

            override fun onLost(network: Network) {
                synchronized(networks) {
                    networks.remove(network)
                    trySend(networks.isNotEmpty())
                }
            }
        }
        trySend(isOnline())
        val request = NetworkRequest.Builder().addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET).build()
        manager.registerNetworkCallback(request, callback)
        awaitClose { manager.unregisterNetworkCallback(callback) }
    }.distinctUntilChanged()
}
