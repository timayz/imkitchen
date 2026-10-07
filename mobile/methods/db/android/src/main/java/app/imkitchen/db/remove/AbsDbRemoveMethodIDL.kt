package app.imkitchen.db.remove

import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodName
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamField
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamModel
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodResultModel
import com.tiktok.sparkling.method.registry.core.base.AbsSparklingIDLMethod
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseParamModel
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseResultModel

/** IDL of `Db.remove`; see `DbStore` for the semantics. */
abstract class AbsDbRemoveMethodIDL :
    AbsSparklingIDLMethod<AbsDbRemoveMethodIDL.Params, AbsDbRemoveMethodIDL.Result>() {

    @IDLMethodName(name = "Db.remove", params = ["collection", "key"], results = ["ok"])
    final override val name: String = "Db.remove"

    @IDLMethodParamModel
    interface Params : IDLMethodBaseParamModel {

        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "collection")
        val collection: String

        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "key")
        val key: String
    }

    @IDLMethodResultModel
    interface Result : IDLMethodBaseResultModel {

        @get:IDLMethodParamField(required = false, isGetter = true, keyPath = "ok")
        @set:IDLMethodParamField(required = false, isGetter = false, keyPath = "ok")
        var ok: Boolean?
    }
}
