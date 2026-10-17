package com.octaviantocan.mymind.update

import java.util.Collections
import java.util.WeakHashMap

/** Something that can have an upload in flight. */
interface UploadWork {
    /** True while a file or a shared item is still being sent or saved. */
    val busy: Boolean
}

/** Tells whether any upload of the app is in flight, so an update does not interrupt it. */
class UploadActivity {
    private val works: MutableSet<UploadWork> = Collections.synchronizedSet(Collections.newSetFromMap(WeakHashMap()))

    /**
     * Starts counting [work]; it is dropped when nothing else holds it.
     *
     * @param work The queue or controller to watch.
     */
    fun register(work: UploadWork) {
        works.add(work)
    }

    /** @return True when any registered work is busy. */
    fun busy(): Boolean = synchronized(works) { works.any { it.busy } }
}
