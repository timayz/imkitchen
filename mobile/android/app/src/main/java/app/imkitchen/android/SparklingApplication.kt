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
import app.imkitchen.db.DbClearMethod
import app.imkitchen.db.DbGetMethod
import app.imkitchen.db.DbListMethod
import app.imkitchen.db.DbPutMethod
import app.imkitchen.db.DbRemoveMethod
import com.tiktok.sparkling.method.media.choosemedia.ChooseMediaMethod
import com.tiktok.sparkling.method.media.downloadfile.DownloadFileMethod
import com.tiktok.sparkling.method.media.savedataurl.SaveDataURLMethod
import com.tiktok.sparkling.method.media.uploadfile.UploadFileMethod
import com.tiktok.sparkling.method.media.uploadimage.UploadImageMethod


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
     * from the splash instead: it rebuilds the kitchen, and a cooking timer
     * that was running is picked up again from storage when the step reopens.
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
            addBehaviors(listOf(
                object : Behavior("input", false) {
                    override fun createUI(context: LynxContext?): LynxUI<*>? {
                        return LynxInputComponent(context)
                    }
                },
                object : Behavior("webview", false) {
                    override fun createUI(context: LynxContext?): LynxUI<*>? {
                        return LynxWebViewComponent(context)
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
        // sparkling-storage (SharedPreferences-backed; holds the session token)
        SparklingBridgeManager.registerIDLMethod(StorageGetItemMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(StorageSetItemMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(StorageRemoveItemMethod::class.java)
        // methods/keep-awake (cooking screen)
        SparklingBridgeManager.registerIDLMethod(KeepAwakeSetEnabledMethod::class.java)
        // methods/timer-alarm (cooking step timer rings through AlarmManager)
        SparklingBridgeManager.registerIDLMethod(TimerAlarmScheduleMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(TimerAlarmCancelMethod::class.java)
        // methods/db (SQLite document store: offline cache + write queue)
        SparklingBridgeManager.registerIDLMethod(DbGetMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(DbPutMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(DbRemoveMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(DbListMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(DbClearMethod::class.java)
        // sparkling-media (recipe photo picker)
        SparklingBridgeManager.registerIDLMethod(ChooseMediaMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(UploadImageMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(UploadFileMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(DownloadFileMethod::class.java)
        SparklingBridgeManager.registerIDLMethod(SaveDataURLMethod::class.java)
    }
}
