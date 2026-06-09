package io.github.berrysoft.tunet;

import android.os.Bundle;
import android.os.StrictMode;

import rs.compio.winio.Activity;

public class MainActivity extends Activity {
    static {
        System.loadLibrary("main");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        StrictMode.ThreadPolicy policy = new StrictMode.ThreadPolicy.Builder().permitNetwork().build();
        StrictMode.setThreadPolicy(policy);
    }
}