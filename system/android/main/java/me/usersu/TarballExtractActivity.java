package me.usersu;

import android.app.Activity;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.widget.TextView;
import android.widget.Button;
import android.widget.LinearLayout;
import android.util.Log;
import java.io.*;
import java.util.zip.GZIPInputStream;
import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream;

public class TarballExtractActivity extends Activity {
    private static final String TAG = "UserSU";
    private static final int PICK_TAR_FILE = 1;
    private TextView statusText;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Create simple UI
        LinearLayout layout = new LinearLayout(this);
        layout.setOrientation(LinearLayout.VERTICAL);
        layout.setPadding(40, 40, 40, 40);
        
        statusText = new TextView(this);
        statusText.setText("UserSU Installer\n\n" +
                "Expected tarball structure:\n" +
                "  bin/su\n" +
                "  lib/libfakeroot.so\n\n" +
                "Tap 'Select Tarball' to choose a .tar.gz file.");
        statusText.setTextSize(14);
        statusText.setPadding(0, 0, 0, 40);
        
        Button selectButton = new Button(this);
        selectButton.setText("Select Tarball");
        selectButton.setOnClickListener(v -> selectTarball());
        
        layout.addView(statusText);
        layout.addView(selectButton);
        
        setContentView(layout);
        
        // Check if already extracted
        File binDir = new File(getFilesDir(), "bin");
        File suBinary = new File(binDir, "su");
        if (suBinary.exists()) {
            statusText.setText("Files already installed!\n\n" +
                    "Location: " + getFilesDir().getAbsolutePath() + "\n" +
                    "Binary: " + suBinary.getAbsolutePath());
        }
    }
    
    private void selectTarball() {
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        intent.setType("*/*");
        startActivityForResult(intent, PICK_TAR_FILE);
    }
    
    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        
        if (requestCode == PICK_TAR_FILE && resultCode == RESULT_OK) {
            if (data != null) {
                Uri uri = data.getData();
                extractTarball(uri);
            }
        }
    }
    
    private void extractTarball(Uri uri) {
        statusText.setText("Extracting tarball...");
        
        new Thread(() -> {
            try {
                InputStream inputStream = getContentResolver().openInputStream(uri);
                if (inputStream == null) {
                    throw new IOException("Cannot open tarball");
                }
                
                // Handle .tar.gz
                InputStream decompressed = inputStream;
                if (uri.toString().endsWith(".gz")) {
                    decompressed = new GZIPInputStream(inputStream);
                }
                
                TarArchiveInputStream tarInput = new TarArchiveInputStream(decompressed);
                
                File filesDir = getFilesDir();
                File rootfsDir = new File(filesDir, "rootfs");
                rootfsDir.mkdirs();
                
                int extracted = 0;
                TarArchiveEntry entry;
                
                while ((entry = (TarArchiveEntry) tarInput.getNextTarEntry()) != null) {
                    String name = entry.getName();
                    
                    // Skip directories
                    if (entry.isDirectory()) {
                        continue;
                    }
                    
                    // Determine output location
                    File outFile;
                    if (name.startsWith("bin/")) {
                        File binDir = new File(filesDir, "bin");
                        binDir.mkdirs();
                        outFile = new File(binDir, new File(name).getName());
                    } else if (name.startsWith("lib/")) {
                        File libDir = new File(filesDir, "lib");
                        libDir.mkdirs();
                        outFile = new File(libDir, new File(name).getName());
                    } else if (name.startsWith("fs/")) {
                        outFile = new File(rootfsDir, name.substring(3));
                        outFile.getParentFile().mkdirs();
                    } else {
                        // Skip unknown files
                        continue;
                    }
                    
                    // Extract file
                    FileOutputStream out = new FileOutputStream(outFile);
                    byte[] buffer = new byte[8192];
                    int read;
                    while ((read = tarInput.read(buffer)) != -1) {
                        out.write(buffer, 0, read);
                    }
                    out.close();
                    
                    // Make executable if in bin/
                    if (name.startsWith("bin/")) {
                        outFile.setExecutable(true, false);
                    }
                    
                    extracted++;
                    Log.i(TAG, "Extracted: " + outFile.getAbsolutePath());
                }
                
                tarInput.close();
                
                final int count = extracted;
                runOnUiThread(() -> {
                    statusText.setText("Extraction complete!\n\n" +
                            "Extracted " + count + " files to:\n" +
                            filesDir.getAbsolutePath() + "\n\n" +
                            "You can now use the su binary.");
                });
                
            } catch (Exception e) {
                Log.e(TAG, "Extraction failed", e);
                runOnUiThread(() -> {
                    statusText.setText("Extraction failed:\n" + e.getMessage() + "\n\n" +
                            "Make sure the tarball has the correct structure:\n" +
                            "  bin/su\n" +
                            "  lib/libfakeroot.so");
                });
            }
        }).start();
    }
}