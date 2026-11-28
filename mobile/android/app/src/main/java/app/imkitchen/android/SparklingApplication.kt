// Copyright (c) 2025 TikTok Pte. Ltd.
// Licensed under the Apache License Version 2.0 that can be found in the
// LICENSE file in the root directory of this source tree.
package app.imkitchen.android

import android.app.Activity
import android.app.Application
import android.content.Intent
import android.os.Bundle

import com.facebook.drawee.backends.pipeline.Fresco
import com.facebook.imagepipeline.core.ImagePipelineConfig
import com.facebook.imagepipeline.memory.PoolConfig
import com.facebook.imagepipeline.memory.PoolFactory
import com.lynx.tasm.behavior.Behavior
import com.lynx.tasm.behavior.LynxContext
import com.lynx.tasm.behavior.ui.LynxUI
import com.tiktok.sparkling.SparklingActivity
import com.tiktok.sparkling.SparklingContextTransferStation
import com.tiktok.sparkling.hybridkit.HybridKit
import com.tiktok.sparkling.hybridkit.config.BaseInfoConfig
import com.tiktok.sparkling.hybridkit.config.SparklingHybridConfig
import com.tiktok.sparkling.hybridkit.config.SparklingLynxConfig
import com.tiktok.sparkling.method.registry.core.SparklingBridgeManager
import com.tiktok.sparkling.method.router.close.RouterCloseMethod
import com.tiktok.sparkling.method.router.open.RouterOpenMethod
import com.tiktok.sparkling.method.router.utils.RouterProvider
import com.tiktok.sparkling.method.storage.getItem.StorageGetItemMethod
import com.tiktok.sparkling.method.storage.removeItem.StorageRemoveItemMethod
import com.tiktok.sparkling.method.storage.setItem.StorageSetItemMethod
import app.imkitchen.keepawake.KeepAwakeSetEnabledMethod
import app.imkitchen.timeralarm.TimerAlarmCancelMethod
import app.imkitchen.timeralarm.TimerAlarmScheduleMethod


class SparklingApplication : Application() {

    override fun onCreate() {
        super.onCreate()
        initFresco()
        initSparkling()
        registerActivityLifecycleCallbacks(RestartAfterProcessDeath())
    }

    /**
     * A `SparklingActivity` finds its page through an in-memory transfer
     * station keyed by an intent extra. When Android recreates the task's top
     * activity after the process was killed in the background, the station is
     * empty and the container renders a bare "Sparkling Page". Start over
     * from the splash instead, which opens the main page again.
     */
    private class RestartAfterProcessDeath : ActivityLifecycleCallbacks {
        override fun onActivityCreated(activity: Activity, savedInstanceState: Bundle?) {
            if (activity !is SparklingActivity || savedInstanceState == null) return
            val id = activity.intent?.getStringExtra(CONTAINER_ID_EXTRA)
            if (id != null && SparklingContextTransferStation.getSparklingContext(id) != null) return
            activity.startActivity(
                Intent(activity, SplashActivity::class.java)
                    .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK)
            )
        }

        override fun onActivityStarted(activity: Activity) {}
        override fun onActivityResumed(activity: Activity) {}
        override fun onActivityPaused(activity: Activity) {}
        override fun onActivityStopped(activity: Activity) {}
        override fun onActivitySaveInstanceState(activity: Activity, outState: Bundle) {}
        override fun onActivityDestroyed(activity: Activity) {}

        private companion object {
            /** The extra `SparklingActivity.onCreate` reads (Sparkling 2.0.1). */
            const val CONTAINER_ID_EXTRA = "SparklingContextContainerId"
        }
    }

    private fun initFresco() {
        val factory = PoolFactory(PoolConfig.newBuilder().build())
        val builder = ImagePipelineConfig.newBuilder(applicationContext).setPoolFactory(factory)
        Fresco.initialize(applicationContext, builder.build())
    }

    private fun initSparkling() {
        initHybridKit()
        initSparklingMethods()
    }


    private fun initHybridKit() {
        HybridKit.init(this)
        val baseInfoConfig = BaseInfoConfig(isDebug = BuildConfig.DEBUG)
        val lynxConfig = SparklingLynxConfig.build(this) {
            // The bundled Lynx has no `<input>`: the host provides it.
            addBehaviors(listOf(
                object : Behavior("input", false) {
                    override fun createUI(context: LynxContext?): LynxUI<*>? {
                        return LynxInputComponent(context)
                    }
                }
            ))
            setTemplateProvider(BuiltinTemplateProvider(this@SparklingApplication))
        }
        val hybridConfig = SparklingHybridConfig.build(baseInfoConfig) {
            setLynxConfig(lynxConfig)
        }
        HybridKit.setHybridConfig(hybridConfig, this)
        HybridKit.initLynxKit()
    }

    // Every Sparkling Method package linked by `sparkling autolink` (see
    // SparklingAutolink.kt) still has to be registered with the bridge here.
    private fun initSparklingMethods() {
        // sparkling-navigation
        SparklingBridgeManager.registerIDLMethod(RouterOpenMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(RouterCloseMethod::class.java)
        RouterProvider.hostRouterDepend = SparklingHostRouterDepend()
        // sparkling-storage (SharedPreferences-backed)
        SparklingBridgeManager.registerIDLMethod(StorageGetItemMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(StorageSetItemMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(StorageRemoveItemMethod::class.java)
        // methods/keep-awake (the screen stays on while cooking)
        SparklingBridgeManager.registerIDLMethod(KeepAwakeSetEnabledMethod::class.java)
        // methods/timer-alarm (a cooking timer rings through AlarmManager)
        SparklingBridgeManager.registerIDLMethod(TimerAlarmScheduleMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(TimerAlarmCancelMethod::class.java)
    }
}
