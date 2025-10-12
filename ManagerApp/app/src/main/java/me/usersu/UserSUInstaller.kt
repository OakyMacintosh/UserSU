package me.usersu

import android.content.Context
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream
import org.apache.commons.compress.compressors.gzip.GzipCompressorInputStream
import org.apache.commons.compress.compressors.xz.XZCompressorInputStream
import java.io.BufferedInputStream
import java.io.File
import java.io.FileOutputStream
import java.io.InputStream

object UserSUInstaller {

    private const val USUD_BINARY = "usud"
    private const val FAKEROOT_LIB = "libfakeroot.so"

    fun extractAndInstall(context: Context, archiveInputStream: InputStream, targetDir: File): Boolean {
        if (!targetDir.exists()) {
            targetDir.mkdirs()
        }

        try {
            BufferedInputStream(archiveInputStream).use { bis ->
                val compressorInputStream = if (isGzip(bis)) {
                    GzipCompressorInputStream(bis)
                } else if (isXz(bis)) {
                    XZCompressorInputStream(bis)
                } else {
                    throw IllegalArgumentException("Unsupported archive format. Only .tar.gz and .su.xz are supported.")
                }

                compressorInputStream.use { cis ->
                    TarArchiveInputStream(cis).use { tarIs ->
                        var entry = tarIs.nextEntry
                        while (entry != null) {
                            val name = entry.name.substringAfterLast('/') // Get just the file name
                            if (name == USUD_BINARY || name == FAKEROOT_LIB) {
                                val outputFile = File(targetDir, name)
                                FileOutputStream(outputFile).use { fos ->
                                    tarIs.copyTo(fos)
                                }
                                // Set executable permission for usud binary
                                if (name == USUD_BINARY) {
                                    outputFile.setExecutable(true, false)
                                }
                            }
                            entry = tarIs.nextEntry
                        }
                    }
                }
            }
            return true
        } catch (e: Exception) {
            e.printStackTrace()
            return false
        }
    }

    private fun isGzip(inputStream: BufferedInputStream): Boolean {
        inputStream.mark(2)
        val b1 = inputStream.read()
        val b2 = inputStream.read()
        inputStream.reset()
        return b1 == 0x1f && b2 == 0x8b
    }

    private fun isXz(inputStream: BufferedInputStream): Boolean {
        inputStream.mark(6)
        val b = ByteArray(6)
        inputStream.read(b)
        inputStream.reset()
        return b[0] == 0xFD.toByte() && b[1] == '7'.toByte() && b[2] == 'Z'.toByte() &&
               b[3] == 'X'.toByte() && b[4] == 'Z'.toByte() && b[5] == 0x00.toByte()
    }
}
