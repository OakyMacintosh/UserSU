package me.usersu;

import android.app.Activity;
import android.content.res.AssetManager;
import android.os.Build;
import android.os.Bundle;
import android.util.Log;
import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;

public class InitActivity extends Activity {
    private static final String TAG = "UserSU";
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        try {
            // Create files directory structure
            File filesDir = getFilesDir();
            File rootfsDir = new File(filesDir, "rootfs");
            rootfsDir.mkdirs();
            
            Log.i(TAG, "Files directory: " + filesDir.getAbsolutePath());
            
            // Extract assets for the device's architecture
            extractAssets();
            
            Log.i(TAG, "Assets extracted successfully");
        } catch (Exception e) {
            Log.e(TAG, "Error during initialization", e);
        }
        
        // Exit immediately
        finish();
    }
    
    private void extractAssets() {
        try {
            File filesDir = getFilesDir();
            String primaryAbi;
            
            // Get primary ABI with version compatibility
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
                String[] abis = Build.SUPPORTED_ABIS;
                primaryAbi = abis[0];
            } else {
                // Fallback for API < 21
                primaryAbi = Build.CPU_ABI;
            }
            
            // Map Android ABI names to asset directory names
            String assetDir;
            if (primaryAbi.contains("arm64") || primaryAbi.contains("aarch64")) {
                assetDir = "aarch64";
            } else if (primaryAbi.contains("armeabi")) {
                assetDir = "armv7l";
            } else {
                Log.w(TAG, "Unsupported ABI: " + primaryAbi);
                return;
            }
            
            Log.i(TAG, "Extracting assets for ABI: " + primaryAbi + " (folder: " + assetDir + ")");
            
            // Extract the entire architecture-specific folder
            extractAssetFolder(assetDir, filesDir);
            
        } catch (Exception e) {
            Log.e(TAG, "Failed to extract assets", e);
        }
    }
    
    private void extractAssetFolder(String assetPath, File targetDir) {
        AssetManager assetManager = getAssets();
        
        try {
            String[] files = assetManager.list(assetPath);
            
            if (files == null || files.length == 0) {
                // This is a file, not a directory
                extractAssetFile(assetPath, targetDir);
                return;
            }
            
            // This is a directory
            File currentDir = new File(targetDir, new File(assetPath).getName());
            if (!currentDir.exists()) {
                currentDir.mkdirs();
            }
            
            // Recursively extract contents
            for (String file : files) {
                String subAssetPath = assetPath + "/" + file;
                extractAssetFolder(subAssetPath, currentDir);
            }
            
        } catch (Exception e) {
            Log.e(TAG, "Error extracting folder: " + assetPath, e);
        }
    }
    
    private void extractAssetFile(String assetPath, File targetDir) {
        AssetManager assetManager = getAssets();
        
        try {
            String fileName = new File(assetPath).getName();
            File outFile = new File(targetDir, fileName);
            
            // Skip placeholder files
            if (fileName.equals("placeholder")) {
                return;
            }
            
            InputStream in = assetManager.open(assetPath);
            OutputStream out = new FileOutputStream(outFile);
            
            byte[] buffer = new byte[8192];
            int read;
            while ((read = in.read(buffer)) != -1) {
                out.write(buffer, 0, read);
            }
            
            in.close();
            out.flush();
            out.close();
            
            // Make binaries executable
            if (assetPath.contains("/bin/")) {
                outFile.setExecutable(true, false);
            }
            
            Log.d(TAG, "Extracted: " + outFile.getAbsolutePath());
            
        } catch (Exception e) {
            Log.e(TAG, "Error extracting file: " + assetPath, e);
        }
    }
}