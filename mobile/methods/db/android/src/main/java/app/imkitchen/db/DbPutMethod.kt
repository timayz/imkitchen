package app.imkitchen.db

import app.imkitchen.db.put.AbsDbPutMethodIDL
import com.tiktok.sparkling.method.registry.core.BridgePlatformType
import com.tiktok.sparkling.method.registry.core.IDLBridgeMethod
import com.tiktok.sparkling.method.registry.core.model.idl.CompletionBlock
import com.tiktok.sparkling.method.registry.core.utils.createXModel
import com.tiktok.sparkling.method.runtime.depend.BridgeBaseRuntime

class DbPutMethod : AbsDbPutMethodIDL() {

    override fun handle(params: Params, callback: CompletionBlock<Result>, type: BridgePlatformType) {
        val context = BridgeBaseRuntime.applicationContext
            ?: return callback.onFailure(IDLBridgeMethod.FAIL, "Context not provided in host")
        val collection = params.collection
        val key = params.key
        val value = params.value
        if (collection.isEmpty() || key.isEmpty() || value.isEmpty()) {
            return callback.onFailure(IDLBridgeMethod.INVALID_PARAM, "Db.put: empty collection/key/value")
        }
        DbStore.execute {
            try {
                DbStore.put(context, collection, key, value)
                callback.onSuccess(Result::class.java.createXModel().apply { this.ok = true })
            } catch (e: Exception) {
                callback.onFailure(IDLBridgeMethod.FAIL, "Db.put failed: $e")
            }
        }
    }
}
