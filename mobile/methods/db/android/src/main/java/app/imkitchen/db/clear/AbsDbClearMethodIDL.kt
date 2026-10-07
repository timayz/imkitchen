package app.imkitchen.db.clear

import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodName
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamField
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamModel
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodResultModel
import com.tiktok.sparkling.method.registry.core.base.AbsSparklingIDLMethod
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseParamModel
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseResultModel

/** IDL of `Db.clear`; see `DbStore` for the semantics. */
abstract class AbsDbClearMethodIDL :
    AbsSparklingIDLMethod<AbsDbClearMethodIDL.Params, AbsDbClearMethodIDL.Result>() {

    @IDLMethodName(name = "Db.clear", params = ["collection"], results = ["ok"])
    final override val name: String = "Db.clear"

    @IDLMethodParamModel
    interface Params : IDLMethodBaseParamModel {

        @get:IDLMethodParamField(required = false, isGetter = true, keyPath = "collection")
        val collection: String?
    }

    @IDLMethodResultModel
    interface Result : IDLMethodBaseResultModel {

        @get:IDLMethodParamField(required = false, isGetter = true, keyPath = "ok")
        @set:IDLMethodParamField(required = false, isGetter = false, keyPath = "ok")
        var ok: Boolean?
    }
}
