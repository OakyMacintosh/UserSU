package dev.usersu

import android.content.Context
import java.io.File

object AndroidApplicationDirectory {
    fun getAppStorageDir(context: Context): File {
        // This will create (if not exists) and return: /data/data/dev.usersu/files
        val appDir = File(context.filesDir, "dev.usersu")
        if (!appDir.exists()) {
            appDir.mkdirs()
        }
        return appDir
    }
}

// setup enviroment variable
fun setupEnvironment(context: Context) {
    val appDir = getAppStorageDir(context)
    System.setProperty("USERSUROOT", appDir.absolutePath)
}.manager
